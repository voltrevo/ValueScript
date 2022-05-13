use std::process::exit;
use std::{path::Path, sync::Arc};
use std::collections::HashSet;
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::rc::Rc;
use std::cell::RefCell;

use swc_ecma_ast::{EsVersion};
use swc_common::{
    errors::{ColorConfig, Handler},
    SourceMap,
};
use swc_ecma_parser::{TsConfig, Syntax};

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

#[derive(Clone)]
enum MappedName {
  Register(String),
  Definition(String),
}

struct ScopeData {
  name_map: HashMap<String, MappedName>,
  parent: Option<Rc<RefCell<ScopeData>>>,
}

type Scope = Rc<RefCell<ScopeData>>;

trait ScopeTrait {
  fn get(&self, name: &String) -> Option<MappedName>;
  fn set(&self, name: String, mapped_name: MappedName);
  fn nest(&self) -> Rc<RefCell<ScopeData>>;
}

impl ScopeTrait for Scope {
  fn get(&self, name: &String) -> Option<MappedName> {
    match self.borrow().name_map.get(name) {
      Some(mapped_name) => Some(mapped_name.clone()),
      None => match &self.borrow().parent {
        Some(parent) => parent.get(name),
        None => None,
      },
    }
  }

  fn set(&self, name: String, mapped_name: MappedName) {
    let old_mapping = self.borrow_mut().name_map.insert(name, mapped_name);

    if old_mapping.is_some() {
      std::panic!("Scope overwrite occurred (not implemented: being permissive about this)");
    }
  }

  fn nest(&self) -> Rc<RefCell<ScopeData>> {
    return Rc::new(RefCell::new(ScopeData {
      name_map: Default::default(),
      parent: Some(self.clone()),
    }));
  }
}

fn init_scope() -> Scope {
  return Rc::new(RefCell::new(ScopeData {
    name_map: Default::default(),
    parent: None,
  }));
}

#[derive(Default)]
struct NameAllocator {
  used_names: HashSet<String>,
  released_names: Vec<String>,
}

impl NameAllocator {
  fn allocate(&mut self, based_on_name: &String) -> String {
    match self.released_names.pop() {
      Some(name) => {
        // FIXME: When reallocating a register we need to ensure we don't read
        // the leftover value
        self.used_names.insert(name.clone());
        return name;
      },
      None => {},
    };

    if !self.used_names.contains(based_on_name) {
      self.used_names.insert(based_on_name.clone());
      return based_on_name.clone();
    }

    return self.allocate_numbered(&(based_on_name.clone() + "_"));
  }

  fn allocate_numbered(&mut self, prefix: &String) -> String {
    match self.released_names.pop() {
      Some(name) => {
        // FIXME: When reallocating a register we need to ensure we don't read
        // the leftover value
        self.used_names.insert(name.clone());
        return name;
      },
      None => {},
    };

    let mut i = 0_u64;

    loop {
      let candidate = prefix.clone() + &i.to_string();

      if !self.used_names.contains(&candidate) {
        self.used_names.insert(candidate.clone());
        return candidate;
      }

      i += 1;
    }
  }

  fn release(&mut self, name: &String) {
    self.used_names.remove(name);
    self.released_names.push(name.clone());
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
      While(_) => std::panic!("Not implemented: While statement"),
      DoWhile(_) => std::panic!("Not implemented: DoWhile statement"),
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
      While(_) => std::panic!("Not implemented: While statement"),
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

struct CompiledExpression {
  value_assembly: String,
  nested_registers: Vec<String>,
}

struct ExpressionCompiler<'a> {
  definition: &'a mut Vec<String>,
  scope: &'a Scope,
  reg_allocator: &'a mut NameAllocator,
}

impl<'a> ExpressionCompiler<'a> {
  fn compile(
    &mut self,
    expr: &swc_ecma_ast::Expr,
    target_register: Option<String>,
  ) -> CompiledExpression {
    use swc_ecma_ast::Expr::*;

    match expr {
      This(_) => {
        return self.inline("%this".to_string(), target_register);
      },
      Array(array_exp) => {
        return self.array_expression(array_exp, target_register);
      },
      Object(_) => std::panic!("Not implemented: Object expression"),
      Fn(_) => std::panic!("Not implemented: Fn expression"),
      Unary(un_exp) => {
        return self.unary_expression(un_exp, target_register);
      },
      Update(_) => std::panic!("Not implemented: Update expression"),
      Bin(bin_exp) => {
        return self.binary_expression(bin_exp, target_register);
      },
      Assign(assign_exp) => {
        return self.assign_expression(assign_exp, target_register);
      },
      Member(_) => std::panic!("Not implemented: Member expression"),
      SuperProp(_) => std::panic!("Not implemented: SuperProp expression"),
      Cond(_) => std::panic!("Not implemented: Cond expression"),
      Call(call_exp) => {
        return self.call_expression(call_exp, target_register);
      },
      New(_) => std::panic!("Not implemented: New expression"),
      Seq(_) => std::panic!("Not implemented: Seq expression"),
      Ident(ident) => {
        return self.identifier(ident, target_register);
      },
      Lit(lit) => {
        return self.literal(lit, target_register);
      },
      Tpl(_) => std::panic!("Not implemented: Tpl expression"),
      TaggedTpl(_) => std::panic!("Not implemented: TaggedTpl expression"),
      Arrow(_) => std::panic!("Not implemented: Arrow expression"),
      Class(_) => std::panic!("Not implemented: Class expression"),
      Yield(_) => std::panic!("Not implemented: Yield expression"),
      MetaProp(_) => std::panic!("Not implemented: MetaProp expression"),
      Await(_) => std::panic!("Not implemented: Await expression"),
      Paren(p) => {
        return self.compile(&*p.expr, target_register);
      },
      JSXMember(_) => std::panic!("Not implemented: JSXMember expression"),
      JSXNamespacedName(_) => std::panic!("Not implemented: JSXNamespacedName expression"),
      JSXEmpty(_) => std::panic!("Not implemented: JSXEmpty expression"),
      JSXElement(_) => std::panic!("Not implemented: JSXElement expression"),
      JSXFragment(_) => std::panic!("Not implemented: JSXFragment expression"),
      TsTypeAssertion(_) => std::panic!("Not implemented: TsTypeAssertion expression"),
      TsConstAssertion(_) => std::panic!("Not implemented: TsConstAssertion expression"),
      TsNonNull(_) => std::panic!("Not implemented: TsNonNull expression"),
      TsAs(_) => std::panic!("Not implemented: TsAs expression"),
      TsInstantiation(_) => std::panic!("Not implemented: TsInstantiation expression"),
      PrivateName(_) => std::panic!("Not implemented: PrivateName expression"),
      OptChain(_) => std::panic!("Not implemented: OptChain expression"),
      Invalid(_) => std::panic!("Not implemented: Invalid expression"),
    };
  }

  fn unary_expression(
    &mut self,
    un_exp: &swc_ecma_ast::UnaryExpr,
    target_register: Option<String>,
  ) -> CompiledExpression {
    let mut nested_registers = Vec::<String>::new();

    let arg = self.compile(
      &un_exp.arg,
      None,
    );

    let mut instr = "  ".to_string();
    instr += get_unary_op_str(un_exp.op);
    instr += " ";
    instr += &arg.value_assembly;

    for used_reg in arg.nested_registers {
      self.reg_allocator.release(&used_reg);
    }

    let target: String = match &target_register {
      None => {
        let res = self.reg_allocator.allocate_numbered(&"_tmp".to_string());
        nested_registers.push(res.clone());
        res
      },
      Some(t) => t.clone(),
    };

    instr += " %";
    instr += &target;

    self.definition.push(instr);

    return CompiledExpression {
      value_assembly: std::format!("%{}", target),
      nested_registers: nested_registers,
    };
  }

  fn binary_expression(
    &mut self,
    bin: &swc_ecma_ast::BinExpr,
    target_register: Option<String>,
  ) -> CompiledExpression {
    let mut nested_registers = Vec::<String>::new();

    let left = self.compile(
      &bin.left,
      None
    );

    let right = self.compile(
      &bin.right,
      None,
    );

    let mut instr = "  ".to_string();
    instr += get_binary_op_str(bin.op);
    instr += " ";
    instr += &left.value_assembly;
    instr += " ";
    instr += &right.value_assembly;

    for used_reg in left.nested_registers {
      self.reg_allocator.release(&used_reg);
    }

    for used_reg in right.nested_registers {
      self.reg_allocator.release(&used_reg);
    }

    let target: String = match &target_register {
      None => {
        let res = self.reg_allocator.allocate_numbered(&"_tmp".to_string());
        nested_registers.push(res.clone());
        res
      },
      Some(t) => t.clone(),
    };

    instr += " %";
    instr += &target;

    self.definition.push(instr);

    return CompiledExpression {
      value_assembly: std::format!("%{}", target),
      nested_registers: nested_registers,
    };
  }

  fn assign_expression(
    &mut self,
    assign_exp: &swc_ecma_ast::AssignExpr,
    target_register: Option<String>,
  ) -> CompiledExpression {
    if assign_exp.op != swc_ecma_ast::AssignOp::Assign {
      std::panic!("Not implemented: compound assignment");
    }

    let assign_name = match &assign_exp.left {
      swc_ecma_ast::PatOrExpr::Expr(_) => std::panic!("Not implemented: assign to expr"),
      swc_ecma_ast::PatOrExpr::Pat(pat) => match &**pat {
        swc_ecma_ast::Pat::Ident(ident) => ident.id.sym.to_string(),
        _ => std::panic!("Not implemented: destructuring"),
      },
    };

    let assign_register = match self.scope.get(&assign_name) {
      None => std::panic!("Unresolved reference"),
      Some(mapping) => match mapping {
        MappedName::Definition(_) => std::panic!("Invalid: assignment to definition"),
        MappedName::Register(reg_name) => reg_name,
      }
    };

    let rhs = self.compile(
      &*assign_exp.right,
      Some(assign_register.clone()),
    );

    // TODO: Consider making two variations of compile, one that takes a target
    // register and one that doesn't. This may simplify things eg by not
    // returning any nested registers when there's a target.
    assert_eq!(rhs.nested_registers.len(), 0);

    if target_register.is_some() {
      let tr = target_register.unwrap();

      let mut instr = "  mov %".to_string();
      instr += &assign_register;
      instr += " %";
      instr += &tr;
      self.definition.push(instr);
    }

    return CompiledExpression {
      value_assembly: "%".to_string() + &assign_register,
      nested_registers: Vec::new(),
    };
  }

  fn array_expression(
    &mut self,
    array_exp: &swc_ecma_ast::ArrayLit,
    target_register: Option<String>,
  ) -> CompiledExpression {
    let mut value_assembly = "[".to_string();
    let mut sub_nested_registers = Vec::<String>::new();

    for i in 0..array_exp.elems.len() {
      match &array_exp.elems[i] {
        None => {
          value_assembly += "void";
        },
        Some(elem) => {
          if elem.spread.is_some() {
            std::panic!("Not implemented: spread expression");
          }

          let mut compiled_elem = self.compile(&*elem.expr, None);
          value_assembly += &compiled_elem.value_assembly;
          sub_nested_registers.append(&mut compiled_elem.nested_registers);
        },
      }

      if i != array_exp.elems.len() - 1 {
        value_assembly += ", ";
      }
    }

    value_assembly += "]";

    return match target_register {
      None => CompiledExpression {
        value_assembly: value_assembly,
        nested_registers: sub_nested_registers,
      },
      Some(tr) => {
        self.definition.push(
          std::format!("  mov {} %{}", value_assembly, tr)
        );

        for reg in sub_nested_registers {
          self.reg_allocator.release(&reg);
        }
        
        CompiledExpression {
          value_assembly: std::format!("%{}", tr),
          nested_registers: Vec::new(),
        }
      },
    };
  }

  fn call_expression(
    &mut self,
    call_exp: &swc_ecma_ast::CallExpr,
    target_register: Option<String>,
  ) -> CompiledExpression {
    let mut nested_registers = Vec::<String>::new();
    let mut sub_nested_registers = Vec::<String>::new();

    let mut callee = match &call_exp.callee {
      swc_ecma_ast::Callee::Expr(expr) => self.compile(&*expr, None),
      _ => std::panic!("Not implemented: non-expression callee"),
    };

    sub_nested_registers.append(&mut callee.nested_registers);

    let mut instr = "  call ".to_string();
    instr += &callee.value_assembly;
    instr += " [";

    for i in 0..call_exp.args.len() {
      let arg = &call_exp.args[i];

      if arg.spread.is_some() {
        std::panic!("Not implemented: argument spreading");
      }

      let mut compiled_arg = self.compile(&*arg.expr, None);
      sub_nested_registers.append(&mut compiled_arg.nested_registers);

      instr += &compiled_arg.value_assembly;

      if i != call_exp.args.len() - 1 {
        instr += ", ";
      }
    }

    instr += "] ";

    let dest = match &target_register {
      Some(tr) => ("%".to_string() + &tr),
      None => {
        let reg = self.reg_allocator.allocate_numbered(&"_tmp".to_string());
        nested_registers.push(reg.clone());

        "%".to_string() + &reg
      },
    };

    instr += &dest;

    self.definition.push(instr);

    for reg in sub_nested_registers {
      self.reg_allocator.release(&reg);
    }

    return CompiledExpression {
      value_assembly: dest,
      nested_registers: nested_registers,
    };
  }

  fn literal(
    &mut self,
    lit: &swc_ecma_ast::Lit,
    target_register: Option<String>,
  ) -> CompiledExpression {
    return self.inline(compile_literal(lit), target_register);
  }

  fn inline(
    &mut self,
    value_assembly: String,
    target_register: Option<String>,
  ) -> CompiledExpression {
    return match target_register {
      None => CompiledExpression {
        value_assembly: value_assembly,
        nested_registers: Vec::new(),
      },
      Some(t) => {
        let mut instr = "  mov ".to_string();
        instr += &value_assembly;
        instr += " %";
        instr += &t;
        self.definition.push(instr);

        CompiledExpression {
          value_assembly: std::format!("%{}", t),
          nested_registers: Vec::new(),
        }
      },
    };
  }

  fn identifier(
    &mut self,
    ident: &swc_ecma_ast::Ident,
    target_register: Option<String>,
  ) -> CompiledExpression {
    let ident_string = ident.sym.to_string();

    let mapped = self.scope.get(&ident_string).expect("Identifier not found in scope");

    let value_assembly = match mapped {
      MappedName::Register(reg) => "%".to_string() + &reg,
      MappedName::Definition(def) => "@".to_string() + &def,
    };

    return self.inline(value_assembly, target_register);
  }
}

fn compile_literal(lit: &swc_ecma_ast::Lit) -> String {
  use swc_ecma_ast::Lit::*;

  return match lit {
    Str(str_) => std::format!("\"{}\"", str_.value), // TODO: Escaping
    Bool(bool_) => bool_.value.to_string(),
    Null(_) => "null".to_string(),
    Num(num) => num.value.to_string(),
    BigInt(_) => std::panic!("Not implemented: BigInt expression"),
    Regex(_) => std::panic!("Not implemented: Regex expression"),
    JSXText(_) => std::panic!("Not implemented: JSXText expression"),
  };
}

fn get_binary_op_str(op: swc_ecma_ast::BinaryOp) -> &'static str {
  use swc_ecma_ast::BinaryOp::*;

  return match op {
    EqEq => "op==",
    NotEq => "op!=",
    EqEqEq => "op===",
    NotEqEq => "op!==",
    Lt => "op<",
    LtEq => "op<=",
    Gt => "op>",
    GtEq => "op>=",
    LShift => "op<<",
    RShift => "op>>",
    ZeroFillRShift => "op>>>",
    Add => "op+",
    Sub => "op-",
    Mul => "op*",
    Div => "op/",
    Mod => "op%",
    BitOr => "op|",
    BitXor => "op^",
    BitAnd => "op&",
    LogicalOr => "op||",
    LogicalAnd => "op&&",
    In => "in",
    InstanceOf => "instanceof",
    Exp => "op**",
    NullishCoalescing => "op??",
  };
}

fn get_unary_op_str(op: swc_ecma_ast::UnaryOp) -> &'static str {
  use swc_ecma_ast::UnaryOp::*;

  return match op {
    Minus => "unary-",
    Plus => "unary+",
    Bang => "op!",
    Tilde => "op~",
    TypeOf => "typeof",
    Void => std::panic!("No matching instruction"),
    Delete => std::panic!("No matching instruction"),
  };
}
