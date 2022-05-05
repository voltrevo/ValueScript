use std::process::exit;
use std::{path::Path, sync::Arc};

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

  std::fs::write("out.vsm", assembly)
    .expect("Failed to write out.vsm");
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

pub fn compile(program: &swc_ecma_ast::Program) -> String {
  let mut compiler = Compiler::default();
  compiler.compile_program(&program);
  std::panic!("Not implemented");
}

#[derive(Default)]
struct Compiler {
  output: Vec<String>,
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
    if module.body.len() != 1 {
      std::panic!("Not implemented: modules that aren't just export default");
    }

    self.compile_module_item(&module.body[0]);
  }

  fn compile_module_item(&mut self, module_item: &swc_ecma_ast::ModuleItem) {
    use swc_ecma_ast::ModuleItem::*;

    match module_item {
      ModuleDecl(module_decl) => self.compile_module_decl(module_decl),
      Stmt(_) => std::panic!("Not supported: module statement"),
    }
  }

  fn compile_module_decl(&mut self, module_decl: &swc_ecma_ast::ModuleDecl) {
    dbg!(module_decl);
    std::panic!("Not implemented");
  }
}
