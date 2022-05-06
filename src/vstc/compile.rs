use std::process::exit;
use std::{path::Path, sync::Arc};
use std::collections::HashSet;
use std::collections::HashMap;

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
    use swc_ecma_ast::ModuleDecl::*;

    match module_decl {
      ExportDefaultDecl(edd) => self.compile_export_default_decl(edd),
      _ => std::panic!("Not implemented: non-default module declaration"),
    }

    dbg!(module_decl);
    std::panic!("Not implemented");
  }

  fn compile_export_default_decl(&mut self, edd: &swc_ecma_ast::ExportDefaultDecl) {
    use swc_ecma_ast::DefaultDecl::*;

    match &edd.decl {
      Fn(fn_) => self.compile_main_fn(fn_),
      _ => std::panic!("Not implemented: Non-function default export"),
    }
  }

  fn compile_main_fn(&mut self, main_fn: &swc_ecma_ast::FnExpr) {
    let mut definition: Vec<String> = Vec::new();
    
    let fn_defn_name = self.definition_allocator.allocate(&match &main_fn.ident {
      Some(ident) => ident.sym.to_string(),
      None => "main".to_string(),
    });

    let mut name_reg_map = HashMap::<String, String>::new();
    let mut reg_allocator = NameAllocator::default();
    let mut param_registers = Vec::<String>::new();

    for p in &main_fn.function.params {
      match &p.pat {
        swc_ecma_ast::Pat::Ident(binding_ident) => {
          let param_name = binding_ident.id.sym.to_string();
          let reg = reg_allocator.allocate(&param_name);
          param_registers.push(reg.clone());
          name_reg_map.insert(param_name, reg);
        },
        _ => std::panic!("Not implemented: parameter destructuring"),
      }
    }

    let mut heading = "@".to_string();
    heading += &fn_defn_name;
    heading += " = function(";

    for i in 0..param_registers.len() {
      heading += "%";
      heading += &param_registers[i];

      if i != param_registers.len() - 1 {
        heading += ", ";
      }
    }

    heading += ") {";

    definition.push(heading);

    let statements = match &main_fn.function.body {
      Some(body) => &body.stmts,
      None => std::panic!(""),
    };

    for statement in statements {
      use swc_ecma_ast::Stmt::*;

      match statement {
        Return(ret_stmt) => match &ret_stmt.arg {
          None => { definition.push("  end".to_string()); }
          Some(_expr) => std::panic!("Not implemented: expressions")
        },
        _ => std::panic!("Not implemented"),
      }
    }

    definition.push("}".to_string());

    dbg!(&definition);

    self.definitions.push(definition);

    // dbg!(main_fn);
    dbg!(fn_defn_name);
    std::panic!("Not implemented");
  }
}

#[derive(Default)]
struct NameAllocator {
  used_names: HashSet<String>,
}

impl NameAllocator {
  fn allocate(&mut self, based_on_name: &String) -> String {
    if !self.used_names.contains(based_on_name) {
      self.used_names.insert(based_on_name.clone());
      return based_on_name.clone();
    }

    return self.allocate_numbered(&(based_on_name.clone() + "_"));
  }

  fn allocate_numbered(&mut self, prefix: &String) -> String {
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
  }
}
