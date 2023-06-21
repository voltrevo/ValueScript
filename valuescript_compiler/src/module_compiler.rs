use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use swc_common::errors::{DiagnosticBuilder, Emitter};
use swc_common::{errors::Handler, FileName, SourceMap, Spanned};
use swc_ecma_ast::EsVersion;
use swc_ecma_parser::{Syntax, TsConfig};

use crate::asm::{
  Class, Definition, DefinitionContent, Function, Instruction, InstructionOrLabel, Lazy, Module,
  Object, Pointer, Register, Value,
};
use crate::diagnostic::{Diagnostic, DiagnosticLevel};
use crate::expression_compiler::{CompiledExpression, ExpressionCompiler};
use crate::function_compiler::{FunctionCompiler, Functionish};
use crate::name_allocator::{ident_from_str, NameAllocator};
use crate::scope::OwnerId;
use crate::scope_analysis::ScopeAnalysis;
use crate::static_eval_expr::static_eval_expr;

struct DiagnosticCollector {
  diagnostics: Arc<Mutex<Vec<Diagnostic>>>,
}

impl Emitter for DiagnosticCollector {
  fn emit(&mut self, db: &DiagnosticBuilder<'_>) {
    match Diagnostic::from_swc(&**db) {
      Some(diagnostic) => self.diagnostics.lock().unwrap().push(diagnostic),
      None => {}
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

  return (result.ok(), diagnostics);
}

#[derive(Default)]
pub struct CompilerOutput {
  pub diagnostics: Vec<Diagnostic>,
  pub module: Module,
}

pub fn compile_program(program: &swc_ecma_ast::Program) -> CompilerOutput {
  let compiler = ModuleCompiler::compile_program(&program);

  return CompilerOutput {
    diagnostics: compiler.diagnostics,
    module: compiler.module,
  };
}

pub fn compile_module(source: &str) -> CompilerOutput {
  let (program_optional, mut diagnostics) = parse(source);

  let mut compiler_output = match program_optional {
    Some(program) => compile_program(&program),
    None => CompilerOutput::default(),
  };

  diagnostics.append(&mut compiler_output.diagnostics);
  compiler_output.diagnostics = diagnostics;

  return compiler_output;
}

#[derive(Default)]
struct ModuleCompiler {
  diagnostics: Vec<Diagnostic>,
  definition_allocator: Rc<RefCell<NameAllocator>>,
  scope_analysis: Rc<ScopeAnalysis>,
  module: Module,
}

impl ModuleCompiler {
  fn todo(&mut self, span: swc_common::Span, message: &str) {
    self.diagnostics.push(Diagnostic {
      level: DiagnosticLevel::InternalError,
      message: format!("TODO: {}", message),
      span,
    });
  }

  fn not_supported(&mut self, span: swc_common::Span, message: &str) {
    self.diagnostics.push(Diagnostic {
      level: DiagnosticLevel::Error,
      message: format!("Not supported: {}", message),
      span,
    });
  }

  fn allocate_defn(&mut self, name: &str) -> Pointer {
    let allocated_name = self
      .definition_allocator
      .borrow_mut()
      .allocate(&name.to_string());

    Pointer {
      name: allocated_name,
    }
  }

  fn allocate_defn_numbered(&mut self, name: &str) -> Pointer {
    let allocated_name = self
      .definition_allocator
      .borrow_mut()
      .allocate_numbered(&name.to_string());

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

        self_.diagnostics.push(Diagnostic {
          level: DiagnosticLevel::Error,
          message: "Scripts are not supported".to_string(),
          span: script.span,
        });

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
      scope_analysis: Rc::new(scope_analysis),
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
      ExportDefaultExpr(_) => self.todo(module_decl.span(), "ExportDefaultExpr declaration"),
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
      Block(block) => self.todo(block.span, "module level Block statement"),
      Empty(_) => {}
      Debugger(debugger) => self.todo(debugger.span, "module level Debugger statement"),
      With(with) => self.todo(with.span, "module level With statement"),
      Return(return_) => self.todo(return_.span, "module level Return statement"),
      Labeled(labeled) => self.todo(labeled.span, "module level Labeled statement"),
      Break(break_) => self.todo(break_.span, "module level Break statement"),
      Continue(continue_) => self.todo(continue_.span, "module level Continue statement"),
      If(if_) => self.todo(if_.span, "module level If statement"),
      Switch(switch) => self.todo(switch.span, "module level Switch statement"),
      Throw(throw) => self.todo(throw.span, "module level Throw statement"),
      Try(try_) => self.todo(try_.span, "module level Try statement"),
      While(while_) => self.todo(while_.span, "module level While statement"),
      DoWhile(do_while) => self.todo(do_while.span, "module level DoWhile statement"),
      For(for_) => self.todo(for_.span, "module level For statement"),
      ForIn(for_in) => self.todo(for_in.span, "module level ForIn statement"),
      ForOf(for_of) => self.todo(for_of.span, "module level ForOf statement"),
      Expr(expr) => self.todo(expr.span, "module level Expr statement"),
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
        if !var_decl.declare {
          self.todo(var_decl.span, "non-declare module level var declaration");
        }
      }
      TsInterface(_) => {}
      TsTypeAlias(_) => {}
      TsEnum(ts_enum) => self.todo(ts_enum.span, "TsEnum declaration"),
      TsModule(ts_module) => self.todo(ts_module.span, "TsModule declaration"),
    };
  }

  fn compile_fn_decl(&mut self, export: bool, fn_: &swc_ecma_ast::FnDecl) {
    let fn_name = fn_.ident.sym.to_string();

    let pointer = match self
      .scope_analysis
      .lookup_value(&OwnerId::Module, &fn_.ident)
    {
      Some(Value::Pointer(p)) => p,
      _ => {
        self.diagnostics.push(Diagnostic {
          level: DiagnosticLevel::InternalError,
          message: format!("Pointer for {} should have been in scope", fn_name),
          span: fn_.ident.span,
        });

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

            let defn = match self.scope_analysis.lookup_value(&OwnerId::Module, ident) {
              Some(Value::Pointer(p)) => p,
              _ => {
                self.diagnostics.push(Diagnostic {
                  level: DiagnosticLevel::InternalError,
                  message: format!("Definition for {} should have been in scope", fn_name),
                  span: ident.span,
                });

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
        self.compile_class(Some(class_name.clone()), Some(&class.ident), &class.class);
      }
      Decl::Fn(fn_) => self.compile_fn_decl(true, fn_),
      Decl::Var(var_decl) => {
        if !var_decl.declare {
          self.todo(
            var_decl.span,
            "non-declare module level var declaration in export",
          );
        }
      }
      Decl::TsInterface(_) => {}
      Decl::TsTypeAlias(_) => {}
      Decl::TsEnum(ts_enum) => self.todo(ts_enum.span, "TsEnum declaration in export"),
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
              self.diagnostics.push(Diagnostic {
                level: DiagnosticLevel::InternalError,
                message: "exporting a non-identifier".to_string(),
                span: named.span,
              });

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
                    true => vec![InstructionOrLabel::Instruction(Instruction::Import(
                      Value::String(src.value.to_string()),
                      Register::return_(),
                    ))],
                    false => vec![
                      InstructionOrLabel::Instruction(Instruction::ImportStar(
                        Value::String(src.value.to_string()),
                        Register::return_(),
                      )),
                      InstructionOrLabel::Instruction(Instruction::Sub(
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
              .lookup_value(&OwnerId::Module, &orig_name)
            {
              Some(Value::Pointer(p)) => Some(p),
              lookup_result => {
                self.diagnostics.push(Diagnostic {
                  level: DiagnosticLevel::InternalError,
                  message: format!(
                    "{} should have been a pointer, but it was {:?}, ref: {:?}",
                    orig_name,
                    lookup_result,
                    self.scope_analysis.refs.get(&orig_name.span)
                  ),
                  span: named.orig.span(),
                });

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
              self.diagnostics.push(Diagnostic {
                level: DiagnosticLevel::InternalError,
                message: "exporting a non-identifier".to_string(),
                span: namespace.span,
              });

              "_todo_export_non_ident".to_string()
            }
          };

          let defn = self.allocate_defn(&namespace_name);

          let src = match &en.src {
            Some(src) => src.value.to_string(),
            None => {
              self.diagnostics.push(Diagnostic {
                level: DiagnosticLevel::InternalError,
                message: "exporting a namespace without a source".to_string(),
                span: namespace.span,
              });

              "_error_export_namespace_without_src".to_string()
            }
          };

          self.module.definitions.push(Definition {
            pointer: defn.clone(),
            content: DefinitionContent::Lazy(Lazy {
              body: vec![InstructionOrLabel::Instruction(Instruction::ImportStar(
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
            .lookup_value(&OwnerId::Module, &named.local)
          {
            Some(Value::Pointer(p)) => p,
            _ => {
              self.diagnostics.push(Diagnostic {
                level: DiagnosticLevel::InternalError,
                message: format!("Imported name {} should have been a pointer", local_name),
                span: named.span,
              });

              self.allocate_defn(local_name.as_str())
            }
          };

          self.module.definitions.push(Definition {
            pointer,
            content: DefinitionContent::Lazy(Lazy {
              body: vec![
                InstructionOrLabel::Instruction(Instruction::ImportStar(
                  Value::String(import_path.clone()),
                  Register::return_(),
                )),
                InstructionOrLabel::Instruction(Instruction::Sub(
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
            .lookup_value(&OwnerId::Module, &default.local)
          {
            Some(Value::Pointer(p)) => p,
            _ => {
              self.diagnostics.push(Diagnostic {
                level: DiagnosticLevel::InternalError,
                message: format!("Imported name {} should have been a pointer", local_name),
                span: default.span,
              });

              self.allocate_defn(local_name.as_str())
            }
          };

          self.module.definitions.push(Definition {
            pointer,
            content: DefinitionContent::Lazy(Lazy {
              body: vec![InstructionOrLabel::Instruction(Instruction::Import(
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
            .lookup_value(&OwnerId::Module, &namespace.local)
          {
            Some(Value::Pointer(p)) => p,
            _ => {
              self.diagnostics.push(Diagnostic {
                level: DiagnosticLevel::InternalError,
                message: format!("Imported name {} should have been a pointer", local_name),
                span: namespace.span,
              });

              self.allocate_defn(local_name.as_str())
            }
          };

          self.module.definitions.push(Definition {
            pointer,
            content: DefinitionContent::Lazy(Lazy {
              body: vec![InstructionOrLabel::Instruction(Instruction::ImportStar(
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
    let (defn, mut diagnostics) = FunctionCompiler::compile(
      defn_pointer,
      fn_name,
      functionish,
      &self.scope_analysis,
      self.definition_allocator.clone(),
    );

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
    let mut methods: Object = Object::default();
    let mut dependent_definitions: Vec<Definition>;

    let defn_name = match ident {
      Some(ident) => match self.scope_analysis.lookup_value(&OwnerId::Module, ident) {
        Some(Value::Pointer(p)) => p,
        _ => {
          self.diagnostics.push(Diagnostic {
            level: DiagnosticLevel::InternalError,
            message: format!("Definition for {} should have been in scope", ident.sym),
            span: class.span, // FIXME: make class_name ident and use that span
          });

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

    let mut member_initializers_fnc = FunctionCompiler::new(
      &self.scope_analysis,
      OwnerId::Span(class.span),
      self.definition_allocator.clone(),
    );

    for class_member in &class.body {
      match class_member {
        swc_ecma_ast::ClassMember::ClassProp(class_prop) => {
          if class_prop.is_static {
            self.todo(class_prop.span, "static props");

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

          let key_asm = ec.fnc.use_(compiled_key);
          let value_asm = ec.fnc.use_(compiled_value);

          ec.fnc
            .push(Instruction::SubMov(key_asm, value_asm, Register::this()));
        }
        swc_ecma_ast::ClassMember::PrivateProp(private_prop) => {
          self.todo(private_prop.span, "private props")
        }
        _ => {}
      }
    }

    let mut member_initializers_assembly = Vec::<InstructionOrLabel>::new();
    member_initializers_assembly.append(&mut member_initializers_fnc.current.body);

    // Include any other definitions that were created by the member initializers
    member_initializers_fnc.process_queue();
    dependent_definitions = std::mem::take(&mut member_initializers_fnc.definitions);

    let mut has_constructor = false;

    for class_member in &class.body {
      match class_member {
        swc_ecma_ast::ClassMember::Constructor(ctor) => {
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
        _ => {}
      }
    }

    if member_initializers_assembly.len() > 0 && !has_constructor {
      let ctor_defn_name = self.allocate_defn(&format!("{}_constructor", defn_name.name));

      constructor = Value::Pointer(ctor_defn_name.clone());
      dependent_definitions.push(Definition {
        pointer: ctor_defn_name.clone(),
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
            swc_ecma_ast::PropName::Computed(computed) => match static_eval_expr(&computed.expr) {
              None => {
                self.todo(
                  computed.span,
                  "Couldn't statically evaluate computed prop name",
                );
                continue;
              }
              Some(value) => value,
            },
            _ => {
              self.todo(method.span, "Non-identifier method name");
              continue;
            }
          };

          let method_defn_name = self.allocate_defn(&ident_from_str(&format!(
            "{}_{}",
            defn_name.name,
            name.to_string()
          )));

          dependent_definitions.append(&mut self.compile_fn(
            method_defn_name.clone(),
            None,
            Functionish::Fn(None, method.function.clone()),
          ));

          methods
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
        methods: Value::Object(Box::new(methods)),
      }),
    });

    self.module.definitions.append(&mut dependent_definitions);

    defn_name
  }
}
