use std::process::exit;
use std::{path::Path, sync::Arc};
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

use super::scope::{Scope, MappedName, init_std_scope, ScopeTrait};
use super::name_allocator::NameAllocator;
use super::function_compiler::FunctionCompiler;

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
  definition_allocator: Rc<RefCell<NameAllocator>>,
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
    let scope = init_std_scope();

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
                    let allocated_name = self.definition_allocator.borrow_mut().allocate(
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
                      self.definition_allocator.borrow_mut().allocate_numbered(&"_anon".to_string())
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
                    self.definition_allocator.borrow_mut().allocate(&fn_.ident.sym.to_string()),
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
          self.definition_allocator.clone(),
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
      Fn(fn_) => self.compile_fn(fn_.ident.sym.to_string(), &fn_.function, self.definition_allocator.clone(), scope),
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
    definition_allocator: Rc<RefCell<NameAllocator>>,
    scope: &Scope,
  ) {
    use swc_ecma_ast::DefaultDecl::*;

    match &edd.decl {
      Fn(fn_) => self.compile_fn(
        fn_name,
        &fn_.function,
        definition_allocator,
        scope,
      ),
      _ => std::panic!("Not implemented: Non-function default export"),
    }
  }

  fn compile_fn(
    &mut self,
    fn_name: String,
    fn_: &swc_ecma_ast::Function,
    definition_allocator: Rc<RefCell<NameAllocator>>,
    parent_scope: &Scope,
  ) {
    self.definitions.push(
      FunctionCompiler::compile(
        fn_name.clone(),
        Some(fn_name),
        fn_,
        definition_allocator,
        parent_scope,
      ),
    );
  }
}
