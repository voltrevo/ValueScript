use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use swc_common::errors::{DiagnosticBuilder, Emitter};
use swc_common::{errors::Handler, FileName, SourceMap, Spanned};
use swc_ecma_ast::EsVersion;
use swc_ecma_parser::{Syntax, TsConfig};

use super::diagnostic::{Diagnostic, DiagnosticLevel};
use super::expression_compiler::{string_literal, CompiledExpression, ExpressionCompiler};
use super::function_compiler::{FunctionCompiler, Functionish};
use super::name_allocator::NameAllocator;
use super::scope::{init_std_scope, MappedName, Scope, ScopeTrait};
use super::scope_analysis::ScopeAnalysis;

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

#[derive(Default, serde::Serialize)]
pub struct CompilerOutput {
  pub diagnostics: Vec<Diagnostic>,
  pub assembly: Vec<String>,
}

pub fn compile_program(program: &swc_ecma_ast::Program) -> CompilerOutput {
  let mut compiler = Compiler::default();
  compiler.compile_program(&program);

  let mut assembly = Vec::<String>::new();
  let mut first = true;

  for def in compiler.definitions {
    if first {
      first = false;
    } else {
      assembly.push("".to_string());
    }

    for line in def {
      assembly.push(line);
    }
  }

  return CompilerOutput {
    diagnostics: compiler.diagnostics,
    assembly,
  };
}

pub fn compile(source: &str) -> CompilerOutput {
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
struct Compiler {
  diagnostics: Vec<Diagnostic>,
  definition_allocator: Rc<RefCell<NameAllocator>>,
  definitions: Vec<Vec<String>>,
}

impl Compiler {
  fn compile_program(&mut self, program: &swc_ecma_ast::Program) {
    use swc_ecma_ast::Program::*;

    match program {
      Module(module) => self.compile_module(module),
      Script(script) => {
        self.diagnostics.push(Diagnostic {
          level: DiagnosticLevel::Error,
          message: "Scripts are not supported".to_string(),
          span: script.span,
        });
      }
    }
  }

  fn compile_module(&mut self, module: &swc_ecma_ast::Module) {
    let mut scope_analysis = ScopeAnalysis::run(module);
    self.diagnostics.append(&mut scope_analysis.diagnostics);
    let scope = init_std_scope();

    use swc_ecma_ast::Decl;
    use swc_ecma_ast::ModuleDecl;
    use swc_ecma_ast::ModuleItem;
    use swc_ecma_ast::Stmt;

    let mut default_export_name = None;

    // Populate scope with top-level declarations
    for module_item in &module.body {
      match module_item {
        ModuleItem::ModuleDecl(module_decl) => match module_decl {
          ModuleDecl::Import(import) => {
            self.diagnostics.push(Diagnostic {
              level: DiagnosticLevel::InternalError,
              message: "TODO: Import module declaration".to_string(),
              span: import.span,
            });
          }
          ModuleDecl::ExportDecl(export_decl) => {
            self.diagnostics.push(Diagnostic {
              level: DiagnosticLevel::InternalError,
              message: "TODO: ExportDecl module declaration".to_string(),
              span: export_decl.span,
            });
          }
          ModuleDecl::ExportNamed(export_named) => {
            self.diagnostics.push(Diagnostic {
              level: DiagnosticLevel::InternalError,
              message: "TODO: ExportNamed module declaration".to_string(),
              span: export_named.span,
            });
          }
          ModuleDecl::ExportDefaultDecl(edd) => {
            match &edd.decl {
              swc_ecma_ast::DefaultDecl::Fn(fn_) => {
                match &fn_.ident {
                  Some(id) => {
                    let allocated_name = self
                      .definition_allocator
                      .borrow_mut()
                      .allocate(&id.sym.to_string());

                    default_export_name = Some(allocated_name.clone());

                    scope.set(id.sym.to_string(), MappedName::Definition(allocated_name));
                  }
                  None => {
                    default_export_name = Some(
                      self
                        .definition_allocator
                        .borrow_mut()
                        .allocate_numbered(&"_anon".to_string()),
                    );
                  }
                };
              }
              swc_ecma_ast::DefaultDecl::Class(class) => {
                self.diagnostics.push(Diagnostic {
                  level: DiagnosticLevel::InternalError,
                  message: "TODO: Class default export".to_string(),
                  span: class.class.span,
                });
              }
              swc_ecma_ast::DefaultDecl::TsInterfaceDecl(ts_interface_decl) => {
                self.diagnostics.push(Diagnostic {
                  level: DiagnosticLevel::InternalError,
                  message: "TODO: TsInterfaceDecl default export".to_string(),
                  span: ts_interface_decl.span,
                });
              }
            };
          }
          ModuleDecl::ExportDefaultExpr(export_default_expr) => {
            self.diagnostics.push(Diagnostic {
              level: DiagnosticLevel::InternalError,
              message: "TODO: ExportDefaultExpr module declaration".to_string(),
              span: export_default_expr.span,
            });
          }
          ModuleDecl::ExportAll(export_all) => {
            self.diagnostics.push(Diagnostic {
              level: DiagnosticLevel::InternalError,
              message: "TODO: ExportAll module declaration".to_string(),
              span: export_all.span,
            });
          }
          ModuleDecl::TsImportEquals(ts_import_equals) => {
            self.diagnostics.push(Diagnostic {
              level: DiagnosticLevel::InternalError,
              message: "TODO: TsImportEquals module declaration".to_string(),
              span: ts_import_equals.span,
            });
          }
          ModuleDecl::TsExportAssignment(ts_export_assignment) => {
            self.diagnostics.push(Diagnostic {
              level: DiagnosticLevel::InternalError,
              message: "TODO: TsExportAssignment module declaration".to_string(),
              span: ts_export_assignment.span,
            });
          }
          ModuleDecl::TsNamespaceExport(ts_namespace_export) => {
            self.diagnostics.push(Diagnostic {
              level: DiagnosticLevel::InternalError,
              message: "TODO: TsNamespaceExport module declaration".to_string(),
              span: ts_namespace_export.span,
            });
          }
        },
        ModuleItem::Stmt(stmt) => match stmt {
          Stmt::Block(block) => {
            self.diagnostics.push(Diagnostic {
              level: DiagnosticLevel::InternalError,
              message: "TODO: module level Block statement".to_string(),
              span: block.span,
            });
          }
          Stmt::Empty(empty) => {
            self.diagnostics.push(Diagnostic {
              level: DiagnosticLevel::InternalError,
              message: "TODO: module level Empty statement".to_string(),
              span: empty.span,
            });
          }
          Stmt::Debugger(debugger) => {
            self.diagnostics.push(Diagnostic {
              level: DiagnosticLevel::InternalError,
              message: "TODO: module level Debugger statement".to_string(),
              span: debugger.span,
            });
          }
          Stmt::With(with) => {
            self.diagnostics.push(Diagnostic {
              level: DiagnosticLevel::InternalError,
              message: "TODO: module level With statement".to_string(),
              span: with.span,
            });
          }
          Stmt::Return(return_) => {
            self.diagnostics.push(Diagnostic {
              level: DiagnosticLevel::InternalError,
              message: "TODO: module level Return statement".to_string(),
              span: return_.span,
            });
          }
          Stmt::Labeled(labeled) => {
            self.diagnostics.push(Diagnostic {
              level: DiagnosticLevel::InternalError,
              message: "TODO: module level Labeled statement".to_string(),
              span: labeled.span,
            });
          }
          Stmt::Break(break_) => {
            self.diagnostics.push(Diagnostic {
              level: DiagnosticLevel::InternalError,
              message: "TODO: module level Break statement".to_string(),
              span: break_.span,
            });
          }
          Stmt::Continue(continue_) => {
            self.diagnostics.push(Diagnostic {
              level: DiagnosticLevel::InternalError,
              message: "TODO: module level Continue statement".to_string(),
              span: continue_.span,
            });
          }
          Stmt::If(if_) => {
            self.diagnostics.push(Diagnostic {
              level: DiagnosticLevel::InternalError,
              message: "TODO: module level If statement".to_string(),
              span: if_.span,
            });
          }
          Stmt::Switch(switch) => {
            self.diagnostics.push(Diagnostic {
              level: DiagnosticLevel::InternalError,
              message: "TODO: module level Switch statement".to_string(),
              span: switch.span,
            });
          }
          Stmt::Throw(throw) => {
            self.diagnostics.push(Diagnostic {
              level: DiagnosticLevel::InternalError,
              message: "TODO: module level Throw statement".to_string(),
              span: throw.span,
            });
          }
          Stmt::Try(try_) => {
            self.diagnostics.push(Diagnostic {
              level: DiagnosticLevel::InternalError,
              message: "TODO: module level Try statement".to_string(),
              span: try_.span,
            });
          }
          Stmt::While(while_) => {
            self.diagnostics.push(Diagnostic {
              level: DiagnosticLevel::InternalError,
              message: "TODO: module level While statement".to_string(),
              span: while_.span,
            });
          }
          Stmt::DoWhile(do_while) => {
            self.diagnostics.push(Diagnostic {
              level: DiagnosticLevel::InternalError,
              message: "TODO: module level DoWhile statement".to_string(),
              span: do_while.span,
            });
          }
          Stmt::For(for_) => {
            self.diagnostics.push(Diagnostic {
              level: DiagnosticLevel::InternalError,
              message: "TODO: module level For statement".to_string(),
              span: for_.span,
            });
          }
          Stmt::ForIn(for_in) => {
            self.diagnostics.push(Diagnostic {
              level: DiagnosticLevel::InternalError,
              message: "TODO: module level ForIn statement".to_string(),
              span: for_in.span,
            });
          }
          Stmt::ForOf(for_of) => {
            self.diagnostics.push(Diagnostic {
              level: DiagnosticLevel::InternalError,
              message: "TODO: module level ForOf statement".to_string(),
              span: for_of.span,
            });
          }
          Stmt::Decl(decl) => {
            match decl {
              Decl::Class(class) => {
                scope.set(
                  class.ident.sym.to_string(),
                  MappedName::Definition(
                    self
                      .definition_allocator
                      .borrow_mut()
                      .allocate(&class.ident.sym.to_string()),
                  ),
                );
              }
              Decl::Fn(fn_) => {
                scope.set(
                  fn_.ident.sym.to_string(),
                  MappedName::Definition(
                    self
                      .definition_allocator
                      .borrow_mut()
                      .allocate(&fn_.ident.sym.to_string()),
                  ),
                );
              }
              Decl::Var(var_decl) => {
                if !var_decl.declare {
                  self.diagnostics.push(Diagnostic {
                    level: DiagnosticLevel::InternalError,
                    message: "TODO: non-declare module level var declaration".to_string(),
                    span: var_decl.span,
                  });
                }
              }
              Decl::TsInterface(_) => {}
              Decl::TsTypeAlias(_) => {}
              Decl::TsEnum(ts_enum) => {
                self.diagnostics.push(Diagnostic {
                  level: DiagnosticLevel::InternalError,
                  message: "TODO: module level TsEnum declaration".to_string(),
                  span: ts_enum.span,
                });
              }
              Decl::TsModule(ts_module) => {
                self.diagnostics.push(Diagnostic {
                  level: DiagnosticLevel::InternalError,
                  message: "TODO: module level TsModule declaration".to_string(),
                  span: ts_module.span,
                });
              }
            };
          }
          Stmt::Expr(expr) => {
            self.diagnostics.push(Diagnostic {
              level: DiagnosticLevel::InternalError,
              message: "TODO: module level Expr statement".to_string(),
              span: expr.span,
            });
          }
        },
      };
    }

    // First compile default
    match default_export_name {
      Some(default_export_name) => {
        for module_item in &module.body {
          match module_item {
            ModuleItem::ModuleDecl(ModuleDecl::ExportDefaultDecl(edd)) => self
              .compile_export_default_decl(
                edd,
                // FIXME: clone() shouldn't be necessary here (we want to move)
                default_export_name.clone(),
                self.definition_allocator.clone(),
                &scope,
              ),
            _ => {}
          }
        }
      }
      None => {
        self.diagnostics.push(Diagnostic {
          level: DiagnosticLevel::InternalError,
          message: "TODO: Modules which don't have a default export name".to_string(),
          span: module.span,
        });
      }
    }

    // Then compile others
    for module_item in &module.body {
      match module_item {
        ModuleItem::ModuleDecl(ModuleDecl::ExportDefaultDecl(_)) => {}
        _ => self.compile_module_item(module_item, &scope),
      }
    }
  }

  fn compile_module_item(&mut self, module_item: &swc_ecma_ast::ModuleItem, scope: &Scope) {
    use swc_ecma_ast::ModuleItem::*;

    match module_item {
      ModuleDecl(module_decl) => self.compile_module_decl(module_decl, scope),
      Stmt(stmt) => self.compile_module_statement(stmt, scope),
    }
  }

  fn compile_module_decl(&mut self, module_decl: &swc_ecma_ast::ModuleDecl, _scope: &Scope) {
    use swc_ecma_ast::ModuleDecl::*;

    match module_decl {
      ExportDefaultDecl(export_default_decl) => {
        self.diagnostics.push(Diagnostic {
          level: DiagnosticLevel::InternalError,
          message: "Default export should be handled elsewhere".to_string(),
          span: export_default_decl.span,
        });
      }
      _ => {
        self.diagnostics.push(Diagnostic {
          level: DiagnosticLevel::InternalError,
          message: "TODO: non-default module declaration".to_string(),
          span: module_decl.span(),
        });
      }
    }
  }

  fn compile_module_statement(&mut self, stmt: &swc_ecma_ast::Stmt, scope: &Scope) {
    use swc_ecma_ast::Stmt::*;

    match stmt {
      Decl(decl) => self.compile_module_level_decl(decl, scope),

      Block(block) => {
        self.diagnostics.push(Diagnostic {
          level: DiagnosticLevel::InternalError,
          message: "TODO: module level Block statement".to_string(),
          span: block.span,
        });
      }
      Empty(empty) => {
        self.diagnostics.push(Diagnostic {
          level: DiagnosticLevel::InternalError,
          message: "TODO: module level Empty statement".to_string(),
          span: empty.span,
        });
      }
      Debugger(debugger) => {
        self.diagnostics.push(Diagnostic {
          level: DiagnosticLevel::InternalError,
          message: "TODO: module level Debugger statement".to_string(),
          span: debugger.span,
        });
      }
      With(with) => {
        self.diagnostics.push(Diagnostic {
          level: DiagnosticLevel::InternalError,
          message: "TODO: module level With statement".to_string(),
          span: with.span,
        });
      }
      Return(return_) => {
        self.diagnostics.push(Diagnostic {
          level: DiagnosticLevel::InternalError,
          message: "TODO: module level Return statement".to_string(),
          span: return_.span,
        });
      }
      Labeled(labeled) => {
        self.diagnostics.push(Diagnostic {
          level: DiagnosticLevel::InternalError,
          message: "TODO: module level Labeled statement".to_string(),
          span: labeled.span,
        });
      }
      Break(break_) => {
        self.diagnostics.push(Diagnostic {
          level: DiagnosticLevel::InternalError,
          message: "TODO: module level Break statement".to_string(),
          span: break_.span,
        });
      }
      Continue(continue_) => {
        self.diagnostics.push(Diagnostic {
          level: DiagnosticLevel::InternalError,
          message: "TODO: module level Continue statement".to_string(),
          span: continue_.span,
        });
      }
      If(if_) => {
        self.diagnostics.push(Diagnostic {
          level: DiagnosticLevel::InternalError,
          message: "TODO: module level If statement".to_string(),
          span: if_.span,
        });
      }
      Switch(switch) => {
        self.diagnostics.push(Diagnostic {
          level: DiagnosticLevel::InternalError,
          message: "TODO: module level Switch statement".to_string(),
          span: switch.span,
        });
      }
      Throw(throw) => {
        self.diagnostics.push(Diagnostic {
          level: DiagnosticLevel::InternalError,
          message: "TODO: module level Throw statement".to_string(),
          span: throw.span,
        });
      }
      Try(try_) => {
        self.diagnostics.push(Diagnostic {
          level: DiagnosticLevel::InternalError,
          message: "TODO: module level Try statement".to_string(),
          span: try_.span,
        });
      }
      While(while_) => {
        self.diagnostics.push(Diagnostic {
          level: DiagnosticLevel::InternalError,
          message: "TODO: module level While statement".to_string(),
          span: while_.span,
        });
      }
      DoWhile(do_while) => {
        self.diagnostics.push(Diagnostic {
          level: DiagnosticLevel::InternalError,
          message: "TODO: module level DoWhile statement".to_string(),
          span: do_while.span,
        });
      }
      For(for_) => {
        self.diagnostics.push(Diagnostic {
          level: DiagnosticLevel::InternalError,
          message: "TODO: module level For statement".to_string(),
          span: for_.span,
        });
      }
      ForIn(for_in) => {
        self.diagnostics.push(Diagnostic {
          level: DiagnosticLevel::InternalError,
          message: "TODO: module level ForIn statement".to_string(),
          span: for_in.span,
        });
      }
      ForOf(for_of) => {
        self.diagnostics.push(Diagnostic {
          level: DiagnosticLevel::InternalError,
          message: "TODO: module level ForOf statement".to_string(),
          span: for_of.span,
        });
      }
      Expr(expr) => {
        self.diagnostics.push(Diagnostic {
          level: DiagnosticLevel::InternalError,
          message: "TODO: module level Expr statement".to_string(),
          span: expr.span,
        });
      }
    };
  }

  fn compile_module_level_decl(&mut self, decl: &swc_ecma_ast::Decl, scope: &Scope) {
    use swc_ecma_ast::Decl::*;

    match decl {
      Class(class) => self.compile_class_decl(class, self.definition_allocator.clone(), scope),
      Fn(fn_) => {
        let fn_name = fn_.ident.sym.to_string();

        let defn = match scope.get_defn(&fn_name) {
          Some(defn) => defn,
          None => {
            self.diagnostics.push(Diagnostic {
              level: DiagnosticLevel::InternalError,
              message: format!("Definition for {} should have been in scope", fn_name),
              span: fn_.ident.span,
            });

            return;
          }
        };

        self.compile_fn(
          defn,
          Some(fn_.ident.sym.to_string()),
          Functionish::Fn(fn_.function.clone()),
          self.definition_allocator.clone(),
          scope,
        )
      }
      Var(var_decl) => {
        if !var_decl.declare {
          self.diagnostics.push(Diagnostic {
            level: DiagnosticLevel::InternalError,
            message: "TODO: non-declare module level var declaration".to_string(),
            span: var_decl.span,
          });
        }
      }
      TsInterface(_) => {}
      TsTypeAlias(_) => {}
      TsEnum(ts_enum) => {
        self.diagnostics.push(Diagnostic {
          level: DiagnosticLevel::InternalError,
          message: "TODO: TsEnum declaration".to_string(),
          span: ts_enum.span,
        });
      }
      TsModule(ts_module) => {
        self.diagnostics.push(Diagnostic {
          level: DiagnosticLevel::InternalError,
          message: "TODO: TsModule declaration".to_string(),
          span: ts_module.span,
        });
      }
    };
  }

  fn compile_export_default_decl(
    &mut self,
    edd: &swc_ecma_ast::ExportDefaultDecl,
    fn_name: String,
    definition_allocator: Rc<RefCell<NameAllocator>>,
    scope: &Scope,
  ) {
    use swc_ecma_ast::DefaultDecl::*;

    match &edd.decl {
      Fn(fn_) => {
        let defn = match scope.get_defn(&fn_name) {
          Some(defn) => defn,
          None => {
            self.diagnostics.push(Diagnostic {
              level: DiagnosticLevel::InternalError,
              message: format!("Definition for {} should have been in scope", fn_name),
              span: edd.span,
            });

            return;
          }
        };

        self.compile_fn(
          defn,
          Some(fn_name),
          Functionish::Fn(fn_.function.clone()),
          definition_allocator,
          scope,
        );
      }
      _ => {
        self.diagnostics.push(Diagnostic {
          level: DiagnosticLevel::InternalError,
          message: "TODO: Non-function default export".to_string(),
          span: edd.span,
        });
      }
    }
  }

  fn compile_fn(
    &mut self,
    defn_name: String,
    fn_name: Option<String>,
    functionish: Functionish,
    definition_allocator: Rc<RefCell<NameAllocator>>,
    parent_scope: &Scope,
  ) {
    let (defn, mut diagnostics) = FunctionCompiler::compile(
      defn_name,
      fn_name,
      functionish,
      definition_allocator,
      parent_scope,
    );

    self.definitions.push(defn);
    self.diagnostics.append(&mut diagnostics);
  }

  fn compile_class_decl(
    &mut self,
    class_decl: &swc_ecma_ast::ClassDecl,
    definition_allocator: Rc<RefCell<NameAllocator>>,
    parent_scope: &Scope,
  ) {
    let mut defn = Vec::<String>::new();

    let class_name = class_decl.ident.sym.to_string();

    let defn_name = match parent_scope.get(&class_name) {
      Some(MappedName::Definition(d)) => d,
      _ => {
        self.diagnostics.push(Diagnostic {
          level: DiagnosticLevel::InternalError,
          message: format!("Definition for {} should have been in scope", class_name),
          span: class_decl.ident.span,
        });

        return;
      }
    };

    let mut constructor_defn_name: Option<String> = None;

    let mut member_initializers_fnc = FunctionCompiler::new(definition_allocator.clone());

    for class_member in &class_decl.class.body {
      match class_member {
        swc_ecma_ast::ClassMember::ClassProp(class_prop) => {
          if class_prop.is_static {
            self.diagnostics.push(Diagnostic {
              level: DiagnosticLevel::InternalError,
              message: "TODO: static props".to_string(),
              span: class_prop.span,
            });

            continue;
          }

          let mut ec = ExpressionCompiler {
            scope: parent_scope,
            fnc: &mut member_initializers_fnc,
          };

          let compiled_key = ec.prop_name(&class_prop.key);

          let compiled_value = match &class_prop.value {
            None =>
            /* CompiledExpression {
              value_assembly: "undefined".to_string(),
              nested_registers: vec![],
            } */
            {
              CompiledExpression::new("undefined".to_string(), vec![])
            }
            Some(expr) => ec.compile(expr, None),
          };

          let key_asm = ec.fnc.use_(compiled_key);
          let value_asm = ec.fnc.use_(compiled_value);

          ec.fnc
            .definition
            .push(format!("  submov {} {} %this", key_asm, value_asm));
        }
        swc_ecma_ast::ClassMember::PrivateProp(private_prop) => {
          self.diagnostics.push(Diagnostic {
            level: DiagnosticLevel::InternalError,
            message: "TODO: private props".to_string(),
            span: private_prop.span,
          });
        }
        _ => {}
      }
    }

    let mut member_initializers_assembly = Vec::<String>::new();
    member_initializers_assembly.append(&mut member_initializers_fnc.definition);

    member_initializers_fnc.process_queue(parent_scope);
    defn.append(&mut member_initializers_fnc.definition);

    let mut has_constructor = false;

    for class_member in &class_decl.class.body {
      match class_member {
        swc_ecma_ast::ClassMember::Constructor(constructor) => {
          has_constructor = true;

          let ctor_defn_name = definition_allocator
            .borrow_mut()
            .allocate(&format!("{}_constructor", class_name));

          self.compile_fn(
            ctor_defn_name.clone(),
            None,
            Functionish::Constructor(member_initializers_assembly.clone(), constructor.clone()),
            definition_allocator.clone(),
            parent_scope,
          );

          constructor_defn_name = Some(ctor_defn_name);
        }
        _ => {}
      }
    }

    if member_initializers_assembly.len() > 0 && !has_constructor {
      let ctor_defn_name = definition_allocator
        .borrow_mut()
        .allocate(&format!("{}_constructor", class_name));

      defn.push(format!("@{} = function() {{", &ctor_defn_name));

      for line in member_initializers_assembly {
        defn.push(line.clone());
      }

      defn.push("}".to_string());
      defn.push("".to_string());

      constructor_defn_name = Some(ctor_defn_name);
    }

    defn.push(format!(
      "@{} = class({}, {{",
      defn_name,
      match constructor_defn_name {
        None => "void".to_string(),
        Some(d) => format!("@{}", d),
      },
    ));

    for class_member in &class_decl.class.body {
      use swc_ecma_ast::ClassMember::*;

      match class_member {
        Constructor(_) => {}
        Method(method) => {
          let name = match &method.key {
            swc_ecma_ast::PropName::Ident(ident) => ident.sym.to_string(),
            _ => {
              self.diagnostics.push(Diagnostic {
                level: DiagnosticLevel::InternalError,
                message: "TODO: Non-identifier method name".to_string(),
                span: method.span,
              });

              continue;
            }
          };

          let method_defn_name = definition_allocator
            .borrow_mut()
            .allocate(&format!("{}_{}", defn_name, name));

          self.compile_fn(
            method_defn_name.clone(),
            None,
            Functionish::Fn(method.function.clone()),
            definition_allocator.clone(),
            parent_scope,
          );

          defn.push(format!(
            "  {}: @{},",
            string_literal(&name),
            method_defn_name,
          ));
        }
        PrivateMethod(private_method) => {
          self.diagnostics.push(Diagnostic {
            level: DiagnosticLevel::InternalError,
            message: "TODO: PrivateMethod".to_string(),
            span: private_method.span,
          });
        }

        // Handled first because they need to be compiled before the
        // constructor, regardless of syntax order
        ClassProp(_) => {}

        PrivateProp(prop) => {
          if prop.value.is_some() {
            self.diagnostics.push(Diagnostic {
              level: DiagnosticLevel::InternalError,
              message: "TODO: class property initializers".to_string(),
              span: prop.span,
            });
          }
        }
        TsIndexSignature(_) => {}
        Empty(_) => {}
        StaticBlock(static_block) => {
          self.diagnostics.push(Diagnostic {
            level: DiagnosticLevel::InternalError,
            message: "TODO: StaticBlock".to_string(),
            span: static_block.span,
          });
        }
      }
    }

    defn.push("})".to_string());

    self.definitions.push(defn);
  }
}
