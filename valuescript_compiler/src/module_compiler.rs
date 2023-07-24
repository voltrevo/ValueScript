use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use swc_common::errors::{DiagnosticBuilder, Emitter};
use swc_common::{errors::Handler, FileName, SourceMap, Spanned};
use swc_ecma_ast::EsVersion;
use swc_ecma_parser::{Syntax, TsConfig};
use valuescript_vm::operations::to_i32;

use crate::asm::{
  Array, Builtin, Class, Definition, DefinitionContent, FnLine, Function, Instruction, Lazy,
  Module, Number, Object, Pointer, Register, Value,
};
use crate::diagnostic::{Diagnostic, DiagnosticContainer, DiagnosticReporter};
use crate::expression_compiler::{value_from_literal, CompiledExpression, ExpressionCompiler};
use crate::function_compiler::{FunctionCompiler, Functionish};
use crate::ident::Ident;
use crate::name_allocator::{ident_from_str, NameAllocator};
use crate::scope::OwnerId;
use crate::scope_analysis::ScopeAnalysis;

struct DiagnosticCollector {
  diagnostics: Arc<Mutex<Vec<Diagnostic>>>,
}

impl Emitter for DiagnosticCollector {
  fn emit(&mut self, db: &DiagnosticBuilder<'_>) {
    if let Some(diagnostic) = Diagnostic::from_swc(db) {
      self.diagnostics.lock().unwrap().push(diagnostic)
    }
  }
}

pub fn parse(source: &str) -> (Option<swc_ecma_ast::Program>, Vec<Diagnostic>) {
  let source_map = Arc::<SourceMap>::default();

  let diagnostics_arc = Arc::new(Mutex::new(Vec::<Diagnostic>::new()));

  let handler = Handler::with_emitter(
    true,
    false,
    Box::new(DiagnosticCollector {
      diagnostics: diagnostics_arc.clone(),
    }),
  );

  let swc_compiler = swc::Compiler::new(source_map.clone());

  let file = source_map.new_source_file(FileName::Anon, source.into());

  let result = swc_compiler.parse_js(
    file,
    &handler,
    EsVersion::Es2022,
    Syntax::Typescript(TsConfig::default()),
    swc::config::IsModule::Bool(true),
    None,
  );

  let mut diagnostics = Vec::<Diagnostic>::new();
  std::mem::swap(&mut diagnostics, &mut *diagnostics_arc.lock().unwrap());

  (result.ok(), diagnostics)
}

#[derive(Default)]
pub struct CompilerOutput {
  pub diagnostics: Vec<Diagnostic>,
  pub module: Module,
}

pub fn compile_program(program: &swc_ecma_ast::Program) -> CompilerOutput {
  let compiler = ModuleCompiler::compile_program(program);

  CompilerOutput {
    diagnostics: compiler.diagnostics,
    module: compiler.module,
  }
}

pub fn compile_module(source: &str) -> CompilerOutput {
  let (program_optional, mut diagnostics) = parse(source);

  let mut compiler_output = match program_optional {
    Some(program) => compile_program(&program),
    None => CompilerOutput::default(),
  };

  diagnostics.append(&mut compiler_output.diagnostics);
  compiler_output.diagnostics = diagnostics;

  compiler_output
}

#[derive(Default)]
pub struct ModuleCompiler {
  pub diagnostics: Vec<Diagnostic>,
  pub definition_allocator: NameAllocator,
  pub scope_analysis: ScopeAnalysis,
  pub constants_map: HashMap<Pointer, Value>,
  pub module: Module,
}

impl DiagnosticContainer for ModuleCompiler {
  fn diagnostics_mut(&mut self) -> &mut Vec<Diagnostic> {
    &mut self.diagnostics
  }
}

impl ModuleCompiler {
  fn allocate_defn(&mut self, name: &str) -> Pointer {
    let allocated_name = self.definition_allocator.allocate(&name.to_string());

    Pointer {
      name: allocated_name,
    }
  }

  fn allocate_defn_numbered(&mut self, name: &str) -> Pointer {
    let allocated_name = self.definition_allocator.allocate_numbered(name);

    Pointer {
      name: allocated_name,
    }
  }

  fn compile_program(program: &swc_ecma_ast::Program) -> Self {
    use swc_ecma_ast::Program::*;

    let module = match program {
      Module(module) => module,
      Script(script) => {
        let mut self_ = Self::default();
        self_.error(script.span, "Scripts are not supported");

        return self_;
      }
    };

    let mut scope_analysis = ScopeAnalysis::run(module);
    let mut scope_analysis_diagnostics = Vec::<Diagnostic>::new();
    std::mem::swap(
      &mut scope_analysis_diagnostics,
      &mut scope_analysis.diagnostics,
    );

    let mut self_ = Self {
      scope_analysis,
      diagnostics: scope_analysis_diagnostics,
      ..Default::default()
    };

    self_.compile_module(module);

    self_
  }

  fn compile_module(&mut self, module: &swc_ecma_ast::Module) {
    for module_item in &module.body {
      self.compile_module_item(module_item);
    }
  }

  fn compile_module_item(&mut self, module_item: &swc_ecma_ast::ModuleItem) {
    use swc_ecma_ast::ModuleItem::*;

    match module_item {
      ModuleDecl(module_decl) => self.compile_module_decl(module_decl),
      Stmt(stmt) => self.compile_module_statement(stmt),
    }
  }

  fn compile_module_decl(&mut self, module_decl: &swc_ecma_ast::ModuleDecl) {
    use swc_ecma_ast::ModuleDecl::*;

    match module_decl {
      Import(import) => self.compile_import(import),
      ExportDecl(ed) => self.compile_export_decl(ed),
      ExportNamed(en) => self.compile_named_export(en),
      ExportDefaultDecl(edd) => self.compile_export_default_decl(edd),
      ExportDefaultExpr(ede) => self.module.export_default = self.compile_expr(&ede.expr),
      ExportAll(_) => self.todo(module_decl.span(), "ExportAll declaration"),
      TsImportEquals(_) => self.not_supported(module_decl.span(), "TsImportEquals declaration"),
      TsExportAssignment(_) => {
        self.not_supported(module_decl.span(), "TsExportAssignment declaration")
      }
      TsNamespaceExport(_) => self.todo(module_decl.span(), "TsNamespaceExport declaration"),
    }
  }

  fn compile_module_statement(&mut self, stmt: &swc_ecma_ast::Stmt) {
    use swc_ecma_ast::Stmt::*;

    match stmt {
      Decl(decl) => self.compile_module_level_decl(decl),
      Block(block) => self.not_supported(block.span, "module level Block statement"),
      Empty(_) => {}
      Debugger(debugger) => self.not_supported(debugger.span, "module level Debugger statement"),
      With(with) => self.not_supported(with.span, "module level With statement"),
      Return(return_) => self.not_supported(return_.span, "module level Return statement"),
      Labeled(labeled) => self.not_supported(labeled.span, "module level Labeled statement"),
      Break(break_) => self.not_supported(break_.span, "module level Break statement"),
      Continue(continue_) => self.not_supported(continue_.span, "module level Continue statement"),
      If(if_) => self.not_supported(if_.span, "module level If statement"),
      Switch(switch) => self.not_supported(switch.span, "module level Switch statement"),
      Throw(throw) => self.not_supported(throw.span, "module level Throw statement"),
      Try(try_) => self.not_supported(try_.span, "module level Try statement"),
      While(while_) => self.not_supported(while_.span, "module level While statement"),
      DoWhile(do_while) => self.not_supported(do_while.span, "module level DoWhile statement"),
      For(for_) => self.not_supported(for_.span, "module level For statement"),
      ForIn(for_in) => self.not_supported(for_in.span, "module level ForIn statement"),
      ForOf(for_of) => self.not_supported(for_of.span, "module level ForOf statement"),
      Expr(expr) => self.not_supported(expr.span, "module level Expr statement"),
    };
  }

  fn compile_module_level_decl(&mut self, decl: &swc_ecma_ast::Decl) {
    use swc_ecma_ast::Decl::*;

    match decl {
      Class(class) => {
        self.compile_class(None, Some(&class.ident), &class.class);
      }
      Fn(fn_) => self.compile_fn_decl(false, fn_),
      Var(var_decl) => {
        self.compile_var_decl(var_decl, false);
      }
      TsInterface(_) => {}
      TsTypeAlias(_) => {}
      TsEnum(ts_enum) => self.compile_enum_decl(false, ts_enum),
      TsModule(ts_module) => self.todo(ts_module.span, "TsModule declaration"),
    };
  }

  fn compile_var_decl(&mut self, var_decl: &swc_ecma_ast::VarDecl, export: bool) {
    if var_decl.declare {
      // Uses the `declare` typescript keyword. Nothing needed to support this.
      return;
    }

    if var_decl.kind != swc_ecma_ast::VarDeclKind::Const {
      // Only `const` variables in the global area. They cannot be mutated, so might as well
      // insist they are `const` for clarity.
      self.not_supported(var_decl.span, "non-const module level variable");
    }

    for decl in &var_decl.decls {
      let ident = match &decl.name {
        swc_ecma_ast::Pat::Ident(bi) => Some(&bi.id),
        _ => {
          self.todo(decl.name.span(), "Module level destructuring");
          None
        }
      };

      let init = match &decl.init {
        Some(_) => &decl.init,
        _ => {
          self.error(decl.init.span(), "const variable without initializer");

          &None
        }
      };

      if let (Some(ident), Some(init)) = (ident, init) {
        let value = self.compile_expr(init);

        let pointer = match self.scope_analysis.lookup(&Ident::from_swc_ident(ident)) {
          Some(name) => match &name.value {
            Value::Pointer(p) => p.clone(),
            _ => {
              self.internal_error(ident.span(), "Expected pointer for module constant");
              continue;
            }
          },
          None => {
            self.internal_error(ident.span(), "Failed to lookup name");
            continue;
          }
        };

        self.constants_map.insert(pointer.clone(), value.clone());

        self.module.definitions.push(Definition {
          pointer: pointer.clone(),
          content: DefinitionContent::Value(value),
        });

        if export {
          self.module.export_star.properties.push((
            Value::String(ident.sym.to_string()),
            Value::Pointer(pointer),
          ));
        }
      }
    }
  }

  fn compile_fn_decl(&mut self, export: bool, fn_: &swc_ecma_ast::FnDecl) {
    let fn_name = fn_.ident.sym.to_string();

    let pointer = match self
      .scope_analysis
      .lookup_value(&OwnerId::Module, &Ident::from_swc_ident(&fn_.ident))
    {
      Some(Value::Pointer(p)) => p,
      _ => {
        self.internal_error(
          fn_.ident.span,
          &format!("Pointer for {} should have been in scope", fn_name),
        );

        return;
      }
    };

    if export {
      self.module.export_star.properties.push((
        Value::String(fn_name.clone()),
        Value::Pointer(pointer.clone()),
      ));
    }

    let mut fn_defns = self.compile_fn(
      pointer,
      Some(fn_name),
      Functionish::Fn(Some(fn_.ident.clone()), fn_.function.clone()),
    );

    self.module.definitions.append(&mut fn_defns);
  }

  fn compile_enum_decl(&mut self, export: bool, ts_enum: &swc_ecma_ast::TsEnumDecl) {
    let pointer = match self
      .scope_analysis
      .lookup_value(&OwnerId::Module, &Ident::from_swc_ident(&ts_enum.id))
    {
      Some(Value::Pointer(p)) => p,
      _ => {
        self.internal_error(
          ts_enum.id.span,
          &format!("Pointer for {} should have been in scope", ts_enum.id.sym),
        );

        return;
      }
    };

    if export {
      self.module.export_star.properties.push((
        Value::String(ts_enum.id.sym.to_string()),
        Value::Pointer(pointer.clone()),
      ));
    }

    let enum_value = self.compile_enum_value(ts_enum);

    self.module.definitions.push(Definition {
      pointer,
      content: DefinitionContent::Value(enum_value),
    });
  }

  fn compile_export_default_decl(&mut self, edd: &swc_ecma_ast::ExportDefaultDecl) {
    use swc_ecma_ast::DefaultDecl;

    match &edd.decl {
      DefaultDecl::Class(class) => {
        let pointer = self.compile_class(None, class.ident.as_ref(), &class.class);
        self.module.export_default = Value::Pointer(pointer);
      }
      DefaultDecl::Fn(fn_) => {
        let (fn_name, defn) = match &fn_.ident {
          Some(ident) => {
            let fn_name = ident.sym.to_string();

            let defn = match self
              .scope_analysis
              .lookup_value(&OwnerId::Module, &Ident::from_swc_ident(ident))
            {
              Some(Value::Pointer(p)) => p,
              _ => {
                self.internal_error(
                  ident.span,
                  &format!("Definition for {} should have been in scope", fn_name),
                );

                return;
              }
            };

            (Some(fn_name), defn)
          }
          None => (None, self.allocate_defn_numbered("_anon")),
        };

        self.module.export_default = Value::Pointer(defn.clone());

        let mut fn_defns = self.compile_fn(
          defn,
          fn_name,
          Functionish::Fn(fn_.ident.clone(), fn_.function.clone()),
        );

        self.module.definitions.append(&mut fn_defns);
      }
      DefaultDecl::TsInterfaceDecl(_) => {
        // Nothing to do
      }
    }
  }

  fn compile_export_decl(&mut self, ed: &swc_ecma_ast::ExportDecl) {
    use swc_ecma_ast::Decl;

    match &ed.decl {
      Decl::Class(class) => {
        let class_name = class.ident.sym.to_string();
        self.compile_class(Some(class_name), Some(&class.ident), &class.class);
      }
      Decl::Fn(fn_) => self.compile_fn_decl(true, fn_),
      Decl::Var(var_decl) => {
        self.compile_var_decl(var_decl, true);
      }
      Decl::TsInterface(_) => {}
      Decl::TsTypeAlias(_) => {}
      Decl::TsEnum(ts_enum) => self.compile_enum_decl(true, ts_enum),
      Decl::TsModule(ts_module) => self.todo(ts_module.span, "TsModule declaration in export"),
    };
  }

  fn compile_named_export(&mut self, en: &swc_ecma_ast::NamedExport) {
    use swc_ecma_ast::ExportSpecifier::*;
    use swc_ecma_ast::ModuleExportName;

    if en.type_only {
      return;
    }

    for specifier in &en.specifiers {
      match specifier {
        Named(named) => {
          if named.is_type_only {
            continue;
          }

          let orig_name = match &named.orig {
            ModuleExportName::Ident(ident) => ident,
            ModuleExportName::Str(_) => {
              self.todo(named.span, "exporting a non-identifier");
              continue;
            }
          };

          let export_name = match &named.exported {
            Some(ModuleExportName::Ident(ident)) => ident.sym.to_string(),
            Some(ModuleExportName::Str(str_)) => {
              self.todo(str_.span, "exporting a non-identifier");
              "_todo_export_non_ident".to_string()
            }
            None => orig_name.sym.to_string(),
          };

          let defn = match &en.src {
            Some(src) => {
              let defn = self.allocate_defn(&export_name);

              self.module.definitions.push(Definition {
                pointer: defn.clone(),
                content: DefinitionContent::Lazy(Lazy {
                  body: match orig_name.sym.to_string() == "default" {
                    true => vec![FnLine::Instruction(Instruction::Import(
                      Value::String(src.value.to_string()),
                      Register::return_(),
                    ))],
                    false => vec![
                      FnLine::Instruction(Instruction::ImportStar(
                        Value::String(src.value.to_string()),
                        Register::return_(),
                      )),
                      FnLine::Instruction(Instruction::Sub(
                        Value::Register(Register::return_()),
                        Value::String(orig_name.sym.to_string()),
                        Register::return_(),
                      )),
                    ],
                  },
                }),
              });

              Some(defn)
            }
            None => match self
              .scope_analysis
              .lookup_value(&OwnerId::Module, &Ident::from_swc_ident(orig_name))
            {
              Some(Value::Pointer(p)) => Some(p),
              lookup_result => {
                self.internal_error(
                  named.orig.span(),
                  &format!(
                    "{} should have been a pointer, but it was {:?}, ref: {:?}",
                    orig_name,
                    lookup_result,
                    self.scope_analysis.refs.get(&orig_name.span)
                  ),
                );

                None
              }
            },
          };

          if let Some(defn) = defn {
            if export_name == "default" {
              self.module.export_default = Value::Pointer(defn);
            } else {
              self
                .module
                .export_star
                .properties
                .push((Value::String(export_name), Value::Pointer(defn)));
            }
          }
        }
        Default(_) => {
          // It's not clear if this can actually be hit. The SWC docs suggest:
          // `export v from 'mod';`
          // but then it refuses to actually parse that.
          self.todo(specifier.span(), "exporting a default module export");
        }
        Namespace(namespace) => {
          let namespace_name = match &namespace.name {
            ModuleExportName::Ident(ident) => ident.sym.to_string(),
            ModuleExportName::Str(_) => {
              self.internal_error(namespace.span, "exporting a non-identifier");
              "_todo_export_non_ident".to_string()
            }
          };

          let defn = self.allocate_defn(&namespace_name);

          let src = match &en.src {
            Some(src) => src.value.to_string(),
            None => {
              self.internal_error(namespace.span, "exporting a namespace without a source");
              "_error_export_namespace_without_src".to_string()
            }
          };

          self.module.definitions.push(Definition {
            pointer: defn.clone(),
            content: DefinitionContent::Lazy(Lazy {
              body: vec![FnLine::Instruction(Instruction::ImportStar(
                Value::String(src),
                Register::return_(),
              ))],
            }),
          });

          if namespace_name == "default" {
            self.module.export_default = Value::Pointer(defn);
          } else {
            self
              .module
              .export_star
              .properties
              .push((Value::String(namespace_name), Value::Pointer(defn)));
          }
        }
      }
    }
  }

  fn compile_import(&mut self, import: &swc_ecma_ast::ImportDecl) {
    if import.type_only {
      return;
    }

    let import_path = import.src.value.to_string();

    for specifier in &import.specifiers {
      use swc_ecma_ast::ImportSpecifier::*;
      use swc_ecma_ast::ModuleExportName;

      match specifier {
        Named(named) => {
          if named.is_type_only {
            continue;
          }

          let local_name = named.local.sym.to_string();

          let external_name = match &named.imported {
            Some(ModuleExportName::Ident(ident)) => ident.sym.to_string(),
            Some(ModuleExportName::Str(str_)) => {
              self.todo(
                str_.span,
                "importing a module export by string (is this a real thing?)",
              );

              "_todo_import_string".to_string()
            }
            None => local_name.clone(),
          };

          let pointer = match self
            .scope_analysis
            .lookup_value(&OwnerId::Module, &Ident::from_swc_ident(&named.local))
          {
            Some(Value::Pointer(p)) => p,
            _ => {
              self.internal_error(
                named.span,
                &format!("Imported name {} should have been a pointer", local_name),
              );

              self.allocate_defn(local_name.as_str())
            }
          };

          self.module.definitions.push(Definition {
            pointer,
            content: DefinitionContent::Lazy(Lazy {
              body: vec![
                FnLine::Instruction(Instruction::ImportStar(
                  Value::String(import_path.clone()),
                  Register::return_(),
                )),
                FnLine::Instruction(Instruction::Sub(
                  Value::Register(Register::return_()),
                  Value::String(external_name),
                  Register::return_(),
                )),
              ],
            }),
          });
        }
        Default(default) => {
          let local_name = default.local.sym.to_string();

          let pointer = match self
            .scope_analysis
            .lookup_value(&OwnerId::Module, &Ident::from_swc_ident(&default.local))
          {
            Some(Value::Pointer(p)) => p,
            _ => {
              self.internal_error(
                default.span,
                &format!("Imported name {} should have been a pointer", local_name),
              );

              self.allocate_defn(local_name.as_str())
            }
          };

          self.module.definitions.push(Definition {
            pointer,
            content: DefinitionContent::Lazy(Lazy {
              body: vec![FnLine::Instruction(Instruction::Import(
                Value::String(import_path.clone()),
                Register::return_(),
              ))],
            }),
          });
        }
        Namespace(namespace) => {
          let local_name = namespace.local.sym.to_string();

          let pointer = match self
            .scope_analysis
            .lookup_value(&OwnerId::Module, &Ident::from_swc_ident(&namespace.local))
          {
            Some(Value::Pointer(p)) => p,
            _ => {
              self.internal_error(
                namespace.span,
                &format!("Imported name {} should have been a pointer", local_name),
              );

              self.allocate_defn(local_name.as_str())
            }
          };

          self.module.definitions.push(Definition {
            pointer,
            content: DefinitionContent::Lazy(Lazy {
              body: vec![FnLine::Instruction(Instruction::ImportStar(
                Value::String(import_path.clone()),
                Register::return_(),
              ))],
            }),
          });
        }
      }
    }
  }

  fn compile_fn(
    &mut self,
    defn_pointer: Pointer,
    fn_name: Option<String>,
    functionish: Functionish,
  ) -> Vec<Definition> {
    let (defn, mut diagnostics) =
      FunctionCompiler::compile(self, defn_pointer, fn_name, functionish);

    self.diagnostics.append(&mut diagnostics);

    defn
  }

  fn compile_class(
    &mut self,
    export_name: Option<String>,
    ident: Option<&swc_ecma_ast::Ident>,
    class: &swc_ecma_ast::Class,
  ) -> Pointer {
    let mut constructor: Value = Value::Void;
    let mut prototype: Object = Object::default();
    let mut static_: Object = Object::default();
    let mut dependent_definitions: Vec<Definition>;

    let defn_name = match ident {
      Some(ident) => match self
        .scope_analysis
        .lookup_value(&OwnerId::Module, &Ident::from_swc_ident(ident))
      {
        Some(Value::Pointer(p)) => p,
        _ => {
          self.internal_error(
            class.span, // FIXME: make class_name ident and use that span
            &format!("Definition for {} should have been in scope", ident.sym),
          );

          self.allocate_defn_numbered("_scope_error")
        }
      },
      None => self.allocate_defn_numbered("_anon"),
    };

    if let Some(export_name) = export_name {
      self.module.export_star.properties.push((
        Value::String(export_name),
        Value::Pointer(defn_name.clone()),
      ));
    }

    let mut member_initializers_fnc = FunctionCompiler::new(self, OwnerId::Span(class.span));

    for class_member in &class.body {
      match class_member {
        swc_ecma_ast::ClassMember::ClassProp(class_prop) => {
          if class_prop.is_static {
            member_initializers_fnc.todo(class_prop.span, "static props");

            continue;
          }

          let mut ec = ExpressionCompiler {
            fnc: &mut member_initializers_fnc,
          };

          let compiled_key = ec.prop_name(&class_prop.key);

          let compiled_value = match &class_prop.value {
            None => CompiledExpression::new(Value::Undefined, vec![]),
            Some(expr) => ec.compile(expr, None),
          };

          ec.fnc.push(Instruction::SubMov(
            compiled_key.value.clone(),
            compiled_value.value.clone(),
            Register::this(),
          ));

          ec.fnc.release_ce(compiled_key);
          ec.fnc.release_ce(compiled_value);
        }
        swc_ecma_ast::ClassMember::PrivateProp(private_prop) => {
          member_initializers_fnc.todo(private_prop.span, "private props")
        }
        _ => {}
      }
    }

    let mut member_initializers_assembly = Vec::<FnLine>::new();
    member_initializers_assembly.append(&mut member_initializers_fnc.current.body);

    // Include any other definitions that were created by the member initializers
    member_initializers_fnc.process_queue();
    dependent_definitions = std::mem::take(&mut member_initializers_fnc.definitions);

    let mut has_constructor = false;

    for class_member in &class.body {
      if let swc_ecma_ast::ClassMember::Constructor(ctor) = class_member {
        has_constructor = true;

        let ctor_defn_name = self.allocate_defn(&format!("{}_constructor", defn_name.name));

        dependent_definitions.append(&mut self.compile_fn(
          ctor_defn_name.clone(),
          None,
          Functionish::Constructor(
            member_initializers_assembly.clone(),
            class.span,
            ctor.clone(),
          ),
        ));

        constructor = Value::Pointer(ctor_defn_name);
      }
    }

    if !member_initializers_assembly.is_empty() && !has_constructor {
      let ctor_defn_name = self.allocate_defn(&format!("{}_constructor", defn_name.name));

      constructor = Value::Pointer(ctor_defn_name.clone());
      dependent_definitions.push(Definition {
        pointer: ctor_defn_name,
        content: DefinitionContent::Function(Function {
          is_generator: false,
          parameters: vec![],
          body: member_initializers_assembly,
        }),
      });
    }

    for class_member in &class.body {
      use swc_ecma_ast::ClassMember::*;

      match class_member {
        Constructor(_) => {}
        Method(method) => {
          let name = match &method.key {
            swc_ecma_ast::PropName::Ident(ident) => Value::String(ident.sym.to_string()),
            swc_ecma_ast::PropName::Computed(computed) => self.compile_expr(&computed.expr),
            _ => {
              self.todo(method.span, "Non-identifier method name");
              continue;
            }
          };

          let method_defn_name =
            self.allocate_defn(&ident_from_str(&format!("{}_{}", defn_name.name, name)));

          dependent_definitions.append(&mut self.compile_fn(
            method_defn_name.clone(),
            None,
            Functionish::Fn(None, method.function.clone()),
          ));

          let dst = match method.is_static {
            false => &mut prototype,
            true => &mut static_,
          };

          dst
            .properties
            .push((name, Value::Pointer(method_defn_name)));
        }
        PrivateMethod(private_method) => self.todo(private_method.span, "PrivateMethod"),

        // Handled first because they need to be compiled before the
        // constructor, regardless of syntax order
        ClassProp(_) => {}

        PrivateProp(prop) => {
          if prop.value.is_some() {
            self.todo(prop.span, "class property initializers");
          }
        }
        TsIndexSignature(_) => {}
        Empty(_) => {}
        StaticBlock(static_block) => {
          self.todo(static_block.span, "StaticBlock");
        }
      }
    }

    self.module.definitions.push(Definition {
      pointer: defn_name.clone(),
      content: DefinitionContent::Class(Class {
        constructor,
        prototype: Value::Object(Box::new(prototype)),
        static_: Value::Object(Box::new(static_)),
      }),
    });

    self.module.definitions.append(&mut dependent_definitions);

    defn_name
  }

  pub fn compile_enum_value(&mut self, ts_enum: &swc_ecma_ast::TsEnumDecl) -> Value {
    let mut properties = Vec::<(Value, Value)>::new();
    let mut next_default_id: Option<f64> = Some(0.0);

    for member in &ts_enum.members {
      let key = match &member.id {
        swc_ecma_ast::TsEnumMemberId::Ident(ident) => ident.sym.to_string(),
        swc_ecma_ast::TsEnumMemberId::Str(str) => str.value.to_string(),
      };

      let init_value = match &member.init {
        Some(init) => {
          let init_value = self.compile_expr(init);

          match self.compile_expr(init) {
            Value::Number(Number(n)) => {
              next_default_id = Some(n + 1.0);
              Some(Value::Number(Number(n)))
            }
            Value::String(_) => Some(init_value),
            _ => None,
          }
        }
        None => None,
      };

      let value = match init_value {
        Some(value) => value,
        None => {
          let id = match next_default_id {
            Some(id) => id,
            None => {
              self.error(member.span, "Missing required initializer");
              0.0
            }
          };

          let value = Value::Number(Number(id));
          next_default_id = Some(id + 1.0);

          value
        }
      };

      properties.push((Value::String(key.clone()), value.clone()));
      properties.push((value, Value::String(key)));
    }

    Value::Object(Box::new(Object { properties }))
  }

  pub fn compile_expr(&mut self, expr: &swc_ecma_ast::Expr) -> Value {
    let symbol_iterator_opt = as_symbol_iterator(expr);

    if let Some(symbol_iterator) = symbol_iterator_opt {
      return symbol_iterator;
    }

    match expr {
      swc_ecma_ast::Expr::Lit(lit) => match value_from_literal(lit) {
        Ok(value) => value,
        Err(msg) => {
          self.internal_error(expr.span(), &format!("Failed to compile literal: {}", msg));
          Value::String("(error)".to_string())
        }
      },
      swc_ecma_ast::Expr::Array(array) => {
        let mut values = Vec::<Value>::new();

        for item in &array.elems {
          values.push(match item {
            Some(item) => {
              if item.spread.is_some() {
                self.todo(expr.span(), "item.spread in static expression");
                return Value::String("(error)".to_string());
              }

              self.compile_expr(&item.expr)
            }
            None => Value::Void,
          });
        }

        Value::Array(Box::new(Array { values }))
      }
      swc_ecma_ast::Expr::Object(object) => {
        let mut properties = Vec::<(Value, Value)>::new();

        for prop in &object.props {
          let (key, value) = match prop {
            swc_ecma_ast::PropOrSpread::Spread(_) => {
              self.todo(prop.span(), "Static spread");
              return Value::String("(error)".to_string());
            }
            swc_ecma_ast::PropOrSpread::Prop(prop) => match &**prop {
              swc_ecma_ast::Prop::Shorthand(_) => {
                self.todo(prop.span(), "Static object shorthand");
                return Value::String("(error)".to_string());
              }
              swc_ecma_ast::Prop::KeyValue(kv) => {
                let key = match &kv.key {
                  swc_ecma_ast::PropName::Ident(ident) => Value::String(ident.sym.to_string()),
                  swc_ecma_ast::PropName::Str(str) => Value::String(str.value.to_string()),
                  swc_ecma_ast::PropName::Num(num) => Value::Number(Number(num.value)),
                  swc_ecma_ast::PropName::Computed(computed) => self.compile_expr(&computed.expr),
                  swc_ecma_ast::PropName::BigInt(bi) => Value::BigInt(bi.value.clone()),
                };

                let value = self.compile_expr(&kv.value);

                (key, value)
              }
              swc_ecma_ast::Prop::Assign(_)
              | swc_ecma_ast::Prop::Getter(_)
              | swc_ecma_ast::Prop::Setter(_) => {
                self.todo(prop.span(), "This type of static prop");
                return Value::String("(error)".to_string());
              }
              swc_ecma_ast::Prop::Method(method) => {
                let key = match &method.key {
                  swc_ecma_ast::PropName::Ident(ident) => Value::String(ident.sym.to_string()),
                  _ => {
                    self.todo(method.key.span(), "Static non-ident prop names");
                    Value::String("(error)".to_string())
                  }
                };

                let fn_ident = match &method.key {
                  swc_ecma_ast::PropName::Ident(ident) => Some(ident.clone()),
                  _ => None,
                };

                let fn_name = fn_ident.clone().map(|ident| ident.sym.to_string());

                let p = match &fn_name {
                  Some(name) => self.allocate_defn(name),
                  None => self.allocate_defn_numbered("_anon"),
                };

                let mut nested_defns = self.compile_fn(
                  p.clone(),
                  fn_name.clone(),
                  Functionish::Fn(fn_ident, method.function.clone()),
                );

                self.module.definitions.append(&mut nested_defns);

                (key, Value::Pointer(p))
              }
            },
          };

          properties.push((key, value));
        }

        Value::Object(Box::new(Object { properties }))
      }
      swc_ecma_ast::Expr::This(_)
      | swc_ecma_ast::Expr::Update(_)
      | swc_ecma_ast::Expr::Assign(_)
      | swc_ecma_ast::Expr::SuperProp(_)
      | swc_ecma_ast::Expr::Call(_)
      | swc_ecma_ast::Expr::New(_) => {
        self.todo(expr.span(), "This type of static expr");
        Value::String("(error)".to_string())
      }
      swc_ecma_ast::Expr::Ident(ident) => match self
        .scope_analysis
        .lookup(&Ident::from_swc_ident(ident))
        .map(|name| name.value.clone())
      {
        Some(Value::Pointer(p)) => self
          .constants_map
          .get(&p)
          .cloned()
          .unwrap_or_else(|| Value::Pointer(p)),
        Some(value) => value,
        None => {
          self.internal_error(ident.span, "Identifier not found");
          Value::String("(error)".to_string())
        }
      },
      swc_ecma_ast::Expr::Fn(fn_) => {
        let fn_name = fn_.ident.clone().map(|ident| ident.sym.to_string());

        let p = match &fn_name {
          Some(name) => self.allocate_defn(name),
          None => self.allocate_defn_numbered("_anon"),
        };

        let mut fn_defns = self.compile_fn(
          p.clone(),
          fn_name,
          Functionish::Fn(fn_.ident.clone(), fn_.function.clone()),
        );

        self.module.definitions.append(&mut fn_defns);

        Value::Pointer(p)
      }
      swc_ecma_ast::Expr::Arrow(arrow) => {
        let p = self.allocate_defn_numbered("_anon");
        let mut fn_defns = self.compile_fn(p.clone(), None, Functionish::Arrow(arrow.clone()));
        self.module.definitions.append(&mut fn_defns);

        Value::Pointer(p)
      }
      swc_ecma_ast::Expr::Class(class) => {
        Value::Pointer(self.compile_class(None, class.ident.as_ref(), &class.class))
      }
      swc_ecma_ast::Expr::TaggedTpl(_)
      | swc_ecma_ast::Expr::Yield(_)
      | swc_ecma_ast::Expr::MetaProp(_)
      | swc_ecma_ast::Expr::Await(_)
      | swc_ecma_ast::Expr::JSXMember(_)
      | swc_ecma_ast::Expr::JSXNamespacedName(_)
      | swc_ecma_ast::Expr::JSXEmpty(_)
      | swc_ecma_ast::Expr::JSXElement(_)
      | swc_ecma_ast::Expr::JSXFragment(_)
      | swc_ecma_ast::Expr::TsInstantiation(_)
      | swc_ecma_ast::Expr::PrivateName(_)
      | swc_ecma_ast::Expr::OptChain(_)
      | swc_ecma_ast::Expr::Invalid(_)
      | swc_ecma_ast::Expr::Member(_)
      | swc_ecma_ast::Expr::Cond(_) => {
        self.todo(expr.span(), "This type of static expr");
        Value::String("(error)".to_string())
      }
      swc_ecma_ast::Expr::Unary(unary) => match unary.op {
        swc_ecma_ast::UnaryOp::Minus => match self.compile_expr(&unary.arg) {
          Value::Number(Number(x)) => Value::Number(Number(-x)),
          Value::BigInt(bi) => Value::BigInt(-bi),
          _ => {
            self.todo(unary.span, "Static eval for this case");
            Value::String("(error)".to_string())
          }
        },
        swc_ecma_ast::UnaryOp::Plus => match self.compile_expr(&unary.arg) {
          Value::Number(Number(x)) => Value::Number(Number(x)),
          Value::BigInt(bi) => Value::BigInt(bi),
          _ => {
            self.todo(unary.span, "Static eval for this case");
            Value::String("(error)".to_string())
          }
        },
        swc_ecma_ast::UnaryOp::Bang => {
          self.todo(expr.span(), "Static eval of ! operator");
          Value::String("(error)".to_string())
        }
        swc_ecma_ast::UnaryOp::Tilde => match self.compile_expr(&unary.arg) {
          Value::Number(Number(x)) => Value::Number(Number(!to_i32(x) as f64)),
          Value::BigInt(bi) => Value::BigInt(!bi),
          _ => {
            self.todo(unary.span, "Static eval for this case");
            Value::String("(error)".to_string())
          }
        },
        swc_ecma_ast::UnaryOp::TypeOf
        | swc_ecma_ast::UnaryOp::Void
        | swc_ecma_ast::UnaryOp::Delete => {
          self.todo(unary.span, "Static eval for this case");
          Value::String("(error)".to_string())
        }
      },
      swc_ecma_ast::Expr::Bin(_) => {
        self.todo(expr.span(), "Static eval of binary operator");
        Value::String("(error)".to_string())
      }
      swc_ecma_ast::Expr::Seq(seq) => {
        let mut last = Value::Void;

        for expr in &seq.exprs {
          last = self.compile_expr(expr);
        }

        last
      }
      swc_ecma_ast::Expr::Tpl(tpl) => 'b: {
        let len = tpl.exprs.len();
        assert_eq!(tpl.quasis.len(), len + 1);

        if len == 0 {
          break 'b Value::String(tpl.quasis[0].raw.to_string());
        }

        self.todo(tpl.span, "Static eval of template literal");
        Value::String("(error)".to_string())
      }
      swc_ecma_ast::Expr::Paren(paren) => self.compile_expr(&paren.expr),
      swc_ecma_ast::Expr::TsTypeAssertion(tta) => self.compile_expr(&tta.expr),
      swc_ecma_ast::Expr::TsConstAssertion(tca) => self.compile_expr(&tca.expr),
      swc_ecma_ast::Expr::TsNonNull(tnn) => self.compile_expr(&tnn.expr),
      swc_ecma_ast::Expr::TsAs(ta) => self.compile_expr(&ta.expr),
    }
  }
}

fn as_symbol_iterator(expr: &swc_ecma_ast::Expr) -> Option<Value> {
  let member_expr = match expr {
    swc_ecma_ast::Expr::Member(member_expr) => member_expr,
    _ => return None,
  };

  match &*member_expr.obj {
    swc_ecma_ast::Expr::Ident(ident) => {
      if ident.sym.to_string() != "Symbol" {
        return None;
      }
    }
    _ => return None,
  };

  match &member_expr.prop {
    swc_ecma_ast::MemberProp::Ident(ident) => {
      if ident.sym.to_string() != "iterator" {
        return None;
      }
    }
    _ => return None,
  }

  Some(Value::Builtin(Builtin {
    name: "SymbolIterator".to_string(),
  }))
}
