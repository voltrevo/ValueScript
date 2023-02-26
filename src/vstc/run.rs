use std::ffi::OsStr;
use std::path::Path;
use std::process::exit;
use std::rc::Rc;
use std::sync::Arc;

use swc_common::errors::DiagnosticBuilder;
use swc_common::errors::Emitter;
use swc_common::errors::Handler;
use swc_common::FileName;
use swc_common::SourceMap;
use swc_ecma_ast::EsVersion;
use swc_ecma_parser::Syntax;
use swc_ecma_parser::TsConfig;

use super::assemble::assemble;
use super::compile::compile;
use super::compile::compile_program;
use super::diagnostic::handle_diagnostics_cli;
use super::diagnostic::Diagnostic;
use super::diagnostic::DiagnosticLevel;
use super::virtual_machine::ValTrait;
use super::virtual_machine::VirtualMachine;

pub fn command(args: &Vec<String>) {
  if args.len() < 3 {
    println!("ERROR: Unrecognized command\n");
    show_help();
    exit(1);
  }

  let mut argpos = 2;

  if args[argpos] == "-h" || args[argpos] == "--help" {
    show_help();
    return;
  }

  let format = match args[argpos].chars().next() {
    Some('-') => {
      let res = format_from_option(&args[argpos]);
      argpos += 1;
      res
    }
    _ => format_from_path(&args[argpos]),
  };

  let file_path = &args[argpos];
  argpos += 1;

  let bytecode = to_bytecode(format, file_path);

  let mut vm = VirtualMachine::new();
  let result = vm.run(&bytecode, &args[argpos..]);

  println!("{}", result);
}

#[derive(serde::Serialize)]
pub struct RunResult {
  pub diagnostics: Vec<Diagnostic>,
  pub output: Result<String, String>,
}

pub fn run(source: &str) -> RunResult {
  let compiler_output = compile(source);

  let mut have_compiler_errors = false;

  for diagnostic in &compiler_output.diagnostics {
    match diagnostic.level {
      DiagnosticLevel::Error => have_compiler_errors = true,
      DiagnosticLevel::InternalError => have_compiler_errors = true,
      _ => (),
    }
  }

  if have_compiler_errors {
    return RunResult {
      diagnostics: compiler_output.diagnostics,
      output: Err("Compile failed".into()),
    };
  }

  let bytecode = assemble(compiler_output.assembly.join("\n").as_str());

  let mut vm = VirtualMachine::new();
  let result = vm.run(&bytecode, &[]);

  return RunResult {
    diagnostics: compiler_output.diagnostics,
    output: Ok(result.codify()),
  };
}

enum RunFormat {
  TypeScript,
  Assembly,
  Bytecode,
}

fn format_from_option(option: &String) -> RunFormat {
  return match option.as_str() {
    "--typescript" => RunFormat::TypeScript,
    "--assembly" => RunFormat::Assembly,
    "--bytecode" => RunFormat::Bytecode,
    _ => std::panic!("Unrecognized option {}", option),
  };
}

fn format_from_path(file_path: &String) -> RunFormat {
  let ext = Path::new(&file_path)
    .extension()
    .and_then(OsStr::to_str)
    .unwrap_or("");

  return match ext {
    "ts" => RunFormat::TypeScript,
    "mts" => RunFormat::TypeScript,
    "js" => RunFormat::TypeScript,
    "mjs" => RunFormat::TypeScript,
    "vsm" => RunFormat::Assembly,
    "vsb" => RunFormat::Bytecode,
    _ => std::panic!("Unrecognized file extension \"{}\"", ext),
  };
}

fn to_bytecode(format: RunFormat, file_path: &String) -> Rc<Vec<u8>> {
  return match format {
    RunFormat::TypeScript => {
      let source = std::fs::read_to_string(file_path).expect("Failed to read file");
      let compiler_output = compile(&source);
      handle_diagnostics_cli(file_path, &compiler_output.diagnostics);

      return assemble(&compiler_output.assembly.join("\n"));
    }

    RunFormat::Assembly => {
      let file_content = std::fs::read_to_string(&file_path)
        .expect(&std::format!("Failed to read file {}", file_path));

      assemble(&file_content)
    }

    RunFormat::Bytecode => {
      Rc::new(std::fs::read(&file_path).expect(&std::format!("Failed to read file {}", file_path)))
    }
  };
}

fn show_help() {
  println!("vstc run");
  println!("");
  println!("Run a ValueScript program");
  println!("");
  println!("USAGE:");
  println!("    vstc run [OPTIONS] <file>");
  println!("");
  println!("OPTIONS:");
  println!("    --assembly");
  println!("            Interpret <file> as assembly");
  println!("");
  println!("    --bytecode");
  println!("            Interpret <file> as bytecode");
  println!("");
  println!("    --typescript");
  println!("            Interpret <file> as typescript");
  println!("");
  println!("NOTE:");
  println!("    <file> will be interpreted based on file extension if not otherwise specified");
}

struct VsEmitter {}

impl Emitter for VsEmitter {
  fn emit(&mut self, db: &DiagnosticBuilder<'_>) {
    // TODO
  }
}
