use std::process::exit;
use std::{path::Path, sync::Arc};
use std::fs::File;
use std::io::prelude::*;

use swc_ecma_ast::{EsVersion};
use swc_common::{
    errors::{ColorConfig, Handler},
    SourceMap,
};
use swc_ecma_parser::{TsConfig, Syntax};

use super::scope::{Scope, MappedName, init_scope, ScopeTrait};
use super::name_allocator::NameAllocator;
use super::expression_compiler::ExpressionCompiler;

pub fn command(args: &Vec<String>) {
  if args.len() != 3 {
    println!("ERROR: Unrecognized command\n");
    show_help();
    exit(1);
  }

  let program = parse(&args[2]);
  let assembly = compile(&program);

  let mut file = File::create("out.vsm").expect("Couldn't create out.vsm");

  for line in assembly {
    file.write_all(line.as_bytes()).expect("Failed to write line");
    file.write_all(b"\n").expect("Failed to write line");
  }
}

fn show_help() {
  println!("vstc compile");
  println!("");
  println!("Compile ValueScript");
  println!("");
  println!("USAGE:");
  println!("    vstc compile <entry point>");
}

pub fn parse(file_path: &String) -> swc_ecma_ast::Program {
  let source_map = Arc::<SourceMap>::default();

  let handler = Handler::with_tty_emitter(
      ColorConfig::Auto,
      true,
      false,
      Some(source_map.clone()),
  );

  let swc_compiler = swc::Compiler::new(source_map.clone());

  let file = source_map
      .load_file(Path::new(&file_path))
      .expect("failed to load file");

  let result = swc_compiler.parse_js(
      file,
      &handler,
      EsVersion::Es2022,
      Syntax::Typescript(TsConfig::default()),
      swc::config::IsModule::Bool(true),
      None,
  );

  return result.expect("Parse failed");
}

pub fn compile(program: &swc_ecma_ast::Program) -> Vec<String> {
  let mut compiler = Compiler::default();
  compiler.compile_program(&program);

  let mut lines = Vec::<String>::new();

  for def in compiler.definitions {
    for line in def {
      lines.push(line);
    }
  }

  return lines;
}

#[derive(Default)]
struct Compiler {
  definition_allocator: NameAllocator,
  definitions: Vec<Vec<String>>,
}

impl Compiler {
  fn compile_program(&mut self, program: &swc_ecma_ast::Program) {
    use swc_ecma_ast::Program::*;

    match program {
      Module(module) => self.compile_module(module),
      Script(_) => std::panic!("Not supported: script"),
    }
  }

  fn compile_module(&mut self, module: &swc_ecma_ast::Module) {
    let scope = init_scope();

    use swc_ecma_ast::ModuleItem;
    use swc_ecma_ast::ModuleDecl;
    use swc_ecma_ast::Stmt;
    use swc_ecma_ast::Decl;

    let mut default_export_name = None;

    // Populate scope with top-level declarations
    for module_item in &module.body {
      match module_item {
        ModuleItem::ModuleDecl(module_decl) => match module_decl {
          ModuleDecl::Import(_) => std::panic!("Not implemented: Import module declaration"),
          ModuleDecl::ExportDecl(_) => std::panic!("Not implemented: ExportDecl module declaration"),
          ModuleDecl::ExportNamed(_) => std::panic!("Not implemented: ExportNamed module declaration"),
          ModuleDecl::ExportDefaultDecl(edd) => {
            match &edd.decl {
              swc_ecma_ast::DefaultDecl::Fn(fn_) => {
                match &fn_.ident {
                  Some(id) => {
                    let allocated_name = self.definition_allocator.allocate(
                      &id.sym.to_string()
                    );

                    default_export_name = Some(allocated_name.clone());

                    scope.set(
                      id.sym.to_string(),
                      MappedName::Definition(allocated_name),
                    );
                  },
                  None => {
                    default_export_name = Some(
                      self.definition_allocator.allocate_numbered(&"_anon".to_string())
                    );
                  },
                };
              },
              _ => std::panic!("Not implemented: Non-function default export"),
            };
          },
          ModuleDecl::ExportDefaultExpr(_) => std::panic!("Not implemented: ExportDefaultExpr module declaration"),
          ModuleDecl::ExportAll(_) => std::panic!("Not implemented: ExportAll module declaration"),
          ModuleDecl::TsImportEquals(_) => std::panic!("Not implemented: TsImportEquals module declaration"),
          ModuleDecl::TsExportAssignment(_) => std::panic!("Not implemented: TsExportAssignment module declaration"),
          ModuleDecl::TsNamespaceExport(_) => std::panic!("Not implemented: TsNamespaceExport module declaration"),
        },
        ModuleItem::Stmt(stmt) => match stmt {
          Stmt::Block(_) => std::panic!("Not implemented: module level Block statement"),
          Stmt::Empty(_) => std::panic!("Not implemented: module level Empty statement"),
          Stmt::Debugger(_) => std::panic!("Not implemented: module level Debugger statement"),
          Stmt::With(_) => std::panic!("Not implemented: module level With statement"),
          Stmt::Return(_) => std::panic!("Not implemented: module level Return statement"),
          Stmt::Labeled(_) => std::panic!("Not implemented: module level Labeled statement"),
          Stmt::Break(_) => std::panic!("Not implemented: module level Break statement"),
          Stmt::Continue(_) => std::panic!("Not implemented: module level Continue statement"),
          Stmt::If(_) => std::panic!("Not implemented: module level If statement"),
          Stmt::Switch(_) => std::panic!("Not implemented: module level Switch statement"),
          Stmt::Throw(_) => std::panic!("Not implemented: module level Throw statement"),
          Stmt::Try(_) => std::panic!("Not implemented: module level Try statement"),
          Stmt::While(_) => std::panic!("Not implemented: module level While statement"),
          Stmt::DoWhile(_) => std::panic!("Not implemented: module level DoWhile statement"),
          Stmt::For(_) => std::panic!("Not implemented: module level For statement"),
          Stmt::ForIn(_) => std::panic!("Not implemented: module level ForIn statement"),
          Stmt::ForOf(_) => std::panic!("Not implemented: module level ForOf statement"),
          Stmt::Decl(decl) => {
            match decl {
              Decl::Class(_) => std::panic!("Not implemented: module level Class declaration"),
              Decl::Fn(fn_) => {
                scope.set(
                  fn_.ident.sym.to_string(),
                  MappedName::Definition(
                    self.definition_allocator.allocate(&fn_.ident.sym.to_string()),
                  ),
                );
              },
              Decl::Var(_) => std::panic!("Not implemented: module level Var declaration"),
              Decl::TsInterface(_) => std::panic!("Not implemented: module level TsInterface declaration"),
              Decl::TsTypeAlias(_) => std::panic!("Not implemented: module level TsTypeAlias declaration"),
              Decl::TsEnum(_) => std::panic!("Not implemented: module level TsEnum declaration"),
              Decl::TsModule(_) => std::panic!("Not implemented: module level TsModule declaration"),
            };
          },
          Stmt::Expr(_) => std::panic!("Not implemented: module level Expr statement"),
        },
      };
    }

    // First compile default
    for module_item in &module.body {
      match module_item {
        ModuleItem::ModuleDecl(
          ModuleDecl::ExportDefaultDecl(edd)
        ) => self.compile_export_default_decl(
          edd,
          // FIXME: clone() shouldn't be necessary here (we want to move)
          default_export_name.clone().expect("Default export name should have been set"),
          &scope,
        ),
        _ => {},
      }
    }

    // Then compile others
    for module_item in &module.body {
      match module_item {
        ModuleItem::ModuleDecl(
          ModuleDecl::ExportDefaultDecl(_)
        ) => {},
        _ => self.compile_module_item(module_item, &scope),
      }
    }
  }

  fn compile_module_item(
    &mut self,
    module_item: &swc_ecma_ast::ModuleItem,
    scope: &Scope,
  ) {
    use swc_ecma_ast::ModuleItem::*;

    match module_item {
      ModuleDecl(module_decl) => self.compile_module_decl(module_decl, scope),
      Stmt(stmt) => self.compile_module_statement(stmt, scope),
    }
  }

  fn compile_module_decl(
    &mut self,
    module_decl: &swc_ecma_ast::ModuleDecl,
    _scope: &Scope,
  ) {
    use swc_ecma_ast::ModuleDecl::*;

    match module_decl {
      ExportDefaultDecl(_) => std::panic!("Default export should be handled elsewhere"),
      _ => std::panic!("Not implemented: non-default module declaration"),
    }
  }

  fn compile_module_statement(
    &mut self,
    stmt: &swc_ecma_ast::Stmt,
    scope: &Scope,
  ) {
    use swc_ecma_ast::Stmt::*;

    match stmt {
      Block(_) => std::panic!("Not implemented: module level Block statement"),
      Empty(_) => std::panic!("Not implemented: module level Empty statement"),
      Debugger(_) => std::panic!("Not implemented: module level Debugger statement"),
      With(_) => std::panic!("Not implemented: module level With statement"),
      Return(_) => std::panic!("Not implemented: module level Return statement"),
      Labeled(_) => std::panic!("Not implemented: module level Labeled statement"),
      Break(_) => std::panic!("Not implemented: module level Break statement"),
      Continue(_) => std::panic!("Not implemented: module level Continue statement"),
      If(_) => std::panic!("Not implemented: module level If statement"),
      Switch(_) => std::panic!("Not implemented: module level Switch statement"),
      Throw(_) => std::panic!("Not implemented: module level Throw statement"),
      Try(_) => std::panic!("Not implemented: module level Try statement"),
      While(_) => std::panic!("Not implemented: module level While statement"),
      DoWhile(_) => std::panic!("Not implemented: module level DoWhile statement"),
      For(_) => std::panic!("Not implemented: module level For statement"),
      ForIn(_) => std::panic!("Not implemented: module level ForIn statement"),
      ForOf(_) => std::panic!("Not implemented: module level ForOf statement"),
      Decl(decl) => self.compile_module_level_decl(decl, scope),
      Expr(_) => std::panic!("Not implemented: module level Expr statement"),
    };
  }

  fn compile_module_level_decl(&mut self, decl: &swc_ecma_ast::Decl, scope: &Scope) {
    use swc_ecma_ast::Decl::*;

    match decl {
      Class(_) => std::panic!("Not implemented: Class declaration"),
      Fn(fn_) => self.compile_fn(fn_.ident.sym.to_string(), &fn_.function, scope),
      Var(_) => std::panic!("Not implemented: Var declaration"),
      TsInterface(_) => std::panic!("Not implemented: TsInterface declaration"),
      TsTypeAlias(_) => std::panic!("Not implemented: TsTypeAlias declaration"),
      TsEnum(_) => std::panic!("Not implemented: TsEnum declaration"),
      TsModule(_) => std::panic!("Not implemented: TsModule declaration"),
    };
  }

  fn compile_export_default_decl(
    &mut self,
    edd: &swc_ecma_ast::ExportDefaultDecl,
    fn_name: String,
    scope: &Scope,
  ) {
    use swc_ecma_ast::DefaultDecl::*;

    match &edd.decl {
      Fn(fn_) => self.compile_fn(
        fn_name,
        &fn_.function,
        scope,
      ),
      _ => std::panic!("Not implemented: Non-function default export"),
    }
  }

  fn compile_fn(
    &mut self,
    fn_name: String,
    fn_: &swc_ecma_ast::Function,
    parent_scope: &Scope,
  ) {
    self.definitions.push(
      FunctionCompiler::compile(fn_name, fn_, parent_scope),
    );
  }
}

struct FunctionCompiler {
  definition: Vec<String>,
  reg_allocator: NameAllocator,
  label_allocator: NameAllocator,
}

impl FunctionCompiler {
  fn new() -> FunctionCompiler {
    let mut reg_allocator = NameAllocator::default();
    reg_allocator.allocate(&"return".to_string());
    reg_allocator.allocate(&"this".to_string());

    return FunctionCompiler {
      definition: Vec::new(),
      reg_allocator: reg_allocator,
      label_allocator: NameAllocator::default(),
    };
  }

  fn compile(
    fn_name: String,
    fn_: &swc_ecma_ast::Function,
    parent_scope: &Scope,
  ) -> Vec<String> {
    let mut self_ = FunctionCompiler::new();
    self_.compile_fn(fn_name, fn_, parent_scope);

    return self_.definition;
  }

  fn compile_fn(
    &mut self,
    fn_name: String,
    fn_: &swc_ecma_ast::Function,
    parent_scope: &Scope,
  ) {
    let scope = parent_scope.nest();

    let mut heading = "@".to_string();
    heading += &fn_name;
    heading += " = function(";

    for i in 0..fn_.params.len() {
      let p = &fn_.params[i];

      match &p.pat {
        swc_ecma_ast::Pat::Ident(binding_ident) => {
          let param_name = binding_ident.id.sym.to_string();
          let reg = self.reg_allocator.allocate(&param_name);

          heading += "%";
          heading += &reg;

          scope.set(
            param_name.clone(),
            MappedName::Register(reg),
          );

          if i != fn_.params.len() - 1 {
            heading += ", ";
          }
        },
        _ => std::panic!("Not implemented: parameter destructuring"),
      }
    }

    heading += ") {";

    self.definition.push(heading);

    let body = fn_.body.as_ref()
      .expect("Not implemented: function without body");
    
    self.populate_fn_scope(body, &scope);
    self.populate_block_scope(body, &scope);

    for i in 0..body.stmts.len() {
      self.statement(
        &body.stmts[i],
        i == body.stmts.len() - 1,
        &scope,
      );
    }

    self.definition.push("}".to_string());
  }

  fn populate_fn_scope(
    &mut self,
    block: &swc_ecma_ast::BlockStmt,
    scope: &Scope,
  ) {
    for statement in &block.stmts {
      self.populate_fn_scope_statement(statement, scope);
    }
  }

  fn populate_fn_scope_statement(
    &mut self,
    statement: &swc_ecma_ast::Stmt,
    scope: &Scope,
  ) {
    use swc_ecma_ast::Stmt::*;

    match statement {
      Block(nested_block) => {
        self.populate_fn_scope(nested_block, scope);
      },
      Empty(_) => {},
      Debugger(_) => {},
      With(_) => std::panic!("Not supported: With statement"),
      Return(_) => {},
      Labeled(_) => std::panic!("Not implemented: Labeled statement"),
      Break(_) => {},
      Continue(_) => {},
      If(if_) => {
        self.populate_fn_scope_statement(&if_.cons, scope);

        for stmt in &if_.alt {
          self.populate_fn_scope_statement(stmt, scope);
        }
      },
      Switch(_) => std::panic!("Not implemented: Switch statement"),
      Throw(_) => {},
      Try(_) => std::panic!("Not implemented: Try statement"),
      While(while_) => {
        self.populate_fn_scope_statement(&while_.body, scope);
      },
      DoWhile(do_while) => {
        self.populate_fn_scope_statement(&do_while.body, scope);
      },
      For(_) => std::panic!("Not implemented: For statement"),
      ForIn(_) => std::panic!("Not implemented: ForIn statement"),
      ForOf(_) => std::panic!("Not implemented: ForOf statement"),
      Decl(decl) => {
        use swc_ecma_ast::Decl::*;

        match decl {
          Class(_) => std::panic!("Not implemented: Class declaration"),
          Fn(_) => std::panic!("Not implemented: Fn declaration"),
          Var(var_decl) => {
            if var_decl.kind == swc_ecma_ast::VarDeclKind::Var {
              for decl in &var_decl.decls {
                match &decl.name {
                  swc_ecma_ast::Pat::Ident(ident) => {
                    let name = ident.id.sym.to_string();

                    scope.set(
                      name.clone(),
                      MappedName::Register(self.reg_allocator.allocate(&name)),
                    );
                  },
                  _ => std::panic!("Not implemented: destructuring"),
                }
              }
            }
          },
          TsInterface(_) => {},
          TsTypeAlias(_) => {},
          TsEnum(_) => std::panic!("Not implemented: TsEnum declaration"),
          TsModule(_) => std::panic!("Not implemented: TsModule declaration"),
        }
      },
      Expr(_) => {},
    };
  }

  fn populate_block_scope(
    &mut self,
    block: &swc_ecma_ast::BlockStmt,
    scope: &Scope,
  ) {
    for statement in &block.stmts {
      use swc_ecma_ast::Stmt::*;

      match statement {
        Block(_) => {},
        Empty(_) => {},
        Debugger(_) => {},
        With(_) => std::panic!("Not supported: With statement"),
        Return(_) => {},
        Labeled(_) => std::panic!("Not implemented: Labeled statement"),
        Break(_) => {},
        Continue(_) => {},
        If(_) => {},
        Switch(_) => {},
        Throw(_) => {},
        Try(_) => {},
        While(_) => {},
        DoWhile(_) => {},
        For(_) => {},
        ForIn(_) => {},
        ForOf(_) => {},
        Decl(decl) => {
          use swc_ecma_ast::Decl::*;
  
          match decl {
            Class(_) => std::panic!("Not implemented: Class declaration"),
            Fn(_) => std::panic!("Not implemented: Fn declaration"),
            Var(var_decl) => {
              if var_decl.kind != swc_ecma_ast::VarDeclKind::Var {
                for decl in &var_decl.decls {
                  match &decl.name {
                    swc_ecma_ast::Pat::Ident(ident) => {
                      let name = ident.id.sym.to_string();
  
                      scope.set(
                        name.clone(),
                        MappedName::Register(self.reg_allocator.allocate(&name)),
                      );
                    },
                    _ => std::panic!("Not implemented: destructuring"),
                  }
                }
              }
            },
            TsInterface(_) => {},
            TsTypeAlias(_) => {},
            TsEnum(_) => std::panic!("Not implemented: TsEnum declaration"),
            TsModule(_) => {},
          }
        },
        Expr(_) => {},
      };
    }
  }

  fn statement(
    &mut self,
    statement: &swc_ecma_ast::Stmt,
    fn_last: bool,
    scope: &Scope,
  ) {
    use swc_ecma_ast::Stmt::*;

    match statement {
      Block(block) => {
        let block_scope = scope.nest();
        self.populate_block_scope(block, &block_scope);

        for stmt in &block.stmts {
          self.statement(stmt, false, &block_scope);
        }

        for mapping in block_scope.borrow().name_map.values() {
          match mapping {
            MappedName::Register(reg) => {
              self.reg_allocator.release(reg);
            },
            MappedName::Definition(_) => {},
          }
        }
      },
      Empty(_) => {},
      Debugger(_) => std::panic!("Not implemented: Debugger statement"),
      With(_) => std::panic!("Not supported: With statement"),

      Return(ret_stmt) => match &ret_stmt.arg {
        None => {
          // TODO: Skip if fn_last
          self.definition.push("  end".to_string());
        },
        Some(expr) => {
          let mut expression_compiler = ExpressionCompiler {
            definition: &mut self.definition,
            scope: scope,
            reg_allocator: &mut self.reg_allocator,
          };

          expression_compiler.compile(expr, Some("return".to_string()));

          if !fn_last {
            self.definition.push("  end".to_string());
          }
        },
      },

      Labeled(_) => std::panic!("Not implemented: Labeled statement"),
      Break(_) => std::panic!("Not implemented: Break statement"),
      Continue(_) => std::panic!("Not implemented: Continue statement"),
      If(if_) => {
        let mut expression_compiler = ExpressionCompiler {
          definition: &mut self.definition,
          scope: scope,
          reg_allocator: &mut self.reg_allocator,
        };

        let condition = expression_compiler.compile(&*if_.test, None);

        for reg in condition.nested_registers {
          self.reg_allocator.release(&reg);
        }

        let cond_reg = self.reg_allocator.allocate_numbered(&"_cond".to_string());

        // TODO: Add negated jmpif instruction to avoid this
        self.definition.push(std::format!(
          "  op! {} %{}",
          condition.value_assembly,
          cond_reg,
        ));

        let else_label = self.label_allocator.allocate_numbered(&"else".to_string());

        let mut jmpif_instr = "  jmpif %".to_string();
        jmpif_instr += &cond_reg;
        jmpif_instr += " :";
        jmpif_instr += &else_label;
        self.definition.push(jmpif_instr);

        self.reg_allocator.release(&cond_reg);

        self.statement(&*if_.cons, false, scope);

        match &if_.alt {
          None => {
            self.definition.push(std::format!("{}:", else_label));
          },
          Some(alt) => {
            let after_else_label = self.label_allocator.allocate_numbered(&"after_else".to_string());
            self.definition.push(std::format!("  jmp :{}", after_else_label));
            self.definition.push(std::format!("{}:", else_label));
            self.statement(&*alt, false, scope);
            self.definition.push(std::format!("{}:", after_else_label));
          }
        }
      },
      Switch(_) => std::panic!("Not implemented: Switch statement"),
      Throw(_) => std::panic!("Not implemented: Throw statement"),
      Try(_) => std::panic!("Not implemented: Try statement"),
      While(while_) => {
        let start_label = self.label_allocator.allocate_numbered(
          &"while".to_string()
        );

        self.definition.push(
          std::format!("{}:", start_label)
        );

        let mut expression_compiler = ExpressionCompiler {
          definition: &mut self.definition,
          scope: scope,
          reg_allocator: &mut self.reg_allocator,
        };

        let condition = expression_compiler.compile(&*while_.test, None);

        for reg in condition.nested_registers {
          self.reg_allocator.release(&reg);
        }

        let cond_reg = self.reg_allocator.allocate_numbered(&"_cond".to_string());

        // TODO: Add negated jmpif instruction to avoid this
        self.definition.push(std::format!(
          "  op! {} %{}",
          condition.value_assembly,
          cond_reg,
        ));

        let end_label = self.label_allocator.allocate_numbered(&"while_end".to_string());

        let mut jmpif_instr = "  jmpif %".to_string();
        jmpif_instr += &cond_reg;
        jmpif_instr += " :";
        jmpif_instr += &end_label;
        self.definition.push(jmpif_instr);

        self.reg_allocator.release(&cond_reg);

        self.statement(&*while_.body, false, scope);
        self.definition.push(std::format!("  jmp :{}", start_label));

        self.definition.push(std::format!("{}:", end_label));
      },
      DoWhile(_) => std::panic!("Not implemented: DoWhile statement"),
      For(_) => std::panic!("Not implemented: For statement"),
      ForIn(_) => std::panic!("Not implemented: ForIn statement"),
      ForOf(_) => std::panic!("Not implemented: ForOf statement"),
      Decl(decl) => {
        self.declaration(decl, scope);
      },
      Expr(expr) => {
        let mut expression_compiler = ExpressionCompiler {
          definition: &mut self.definition,
          scope: scope,
          reg_allocator: &mut self.reg_allocator,
        };

        let compiled = expression_compiler.compile(&*expr.expr, None);

        for reg in compiled.nested_registers {
          self.reg_allocator.release(&reg);
        }
      },
    }
  }

  fn declaration(
    &mut self,
    decl: &swc_ecma_ast::Decl,
    scope: &Scope,
  ) {
    use swc_ecma_ast::Decl::*;

    match decl {
      Class(_) => std::panic!("Not implemented: Class declaration"),
      Fn(_) => std::panic!("Not implemented: Fn declaration"),
      Var(var_decl) => self.var_declaration(var_decl, scope),
      TsInterface(_) => std::panic!("Not implemented: TsInterface declaration"),
      TsTypeAlias(_) => std::panic!("Not implemented: TsTypeAlias declaration"),
      TsEnum(_) => std::panic!("Not implemented: TsEnum declaration"),
      TsModule(_) => std::panic!("Not implemented: TsModule declaration"),
    };
  }

  fn var_declaration(
    &mut self,
    var_decl: &swc_ecma_ast::VarDecl,
    scope: &Scope,
  ) {
    for decl in &var_decl.decls {
      match &decl.init {
        Some(expr) => {
          let mut expr_compiler = ExpressionCompiler {
            definition: &mut self.definition,
            scope: scope,
            reg_allocator: &mut self.reg_allocator,
          };

          let name = match &decl.name {
            swc_ecma_ast::Pat::Ident(ident) => ident.id.sym.to_string(),
            _ => std::panic!("Not implemented: destructuring"),
          };

          let target_register = match scope.get(&name) {
            Some(MappedName::Register(reg_name)) => reg_name,
            _ => std::panic!("var decl should always get mapped to a register during scan"),
          };

          expr_compiler.compile(expr, Some(target_register));
        },
        None => {},
      }
    }
  }
}
