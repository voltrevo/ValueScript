use std::fs;
use std::rc::Rc;
use std::{ffi::OsStr, path::Path, process::exit};

use valuescript_compiler::{assemble, compile, parse_module};
use valuescript_vm::vs_value::Val;
use valuescript_vm::{Bytecode, VirtualMachine};

use crate::resolve_entry_path::resolve_entry_path;

use super::handle_diagnostics_cli::handle_diagnostics_cli;

pub fn run_command(args: &Vec<String>) {
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

  let bytecode = Rc::new(to_bytecode(format, file_path));

  let mut vm = VirtualMachine::default();

  let val_args: Vec<Val> = args[argpos..]
    .iter()
    .map(|a| Val::String(Rc::from(a.clone())))
    .collect();

  match vm.run(bytecode, None, &val_args) {
    Ok(result) => {
      println!("{}", result.pretty());
    }
    Err(err) => {
      println!("Uncaught exception: {}", err.pretty());
      exit(1);
    }
  }
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

  match ext {
    "ts" => RunFormat::TypeScript,
    "mts" => RunFormat::TypeScript,
    "js" => RunFormat::TypeScript,
    "mjs" => RunFormat::TypeScript,
    "vsm" => RunFormat::Assembly,
    "vsb" => RunFormat::Bytecode,
    _ => std::panic!("Unrecognized file extension \"{}\"", ext),
  }
}

fn to_bytecode(format: RunFormat, file_path: &String) -> Bytecode {
  Bytecode::new(match format {
    RunFormat::TypeScript => {
      let resolved_entry_path = resolve_entry_path(file_path);

      let compile_result = compile(resolved_entry_path, |path| {
        std::fs::read_to_string(path).map_err(|err| err.to_string())
      });

      for (path, diagnostics) in compile_result.diagnostics.iter() {
        handle_diagnostics_cli(&path.path, diagnostics);
      }

      assemble(
        &compile_result
          .module
          .expect("Should have exited if module is None"),
      )
    }

    RunFormat::Assembly => {
      let file_content = std::fs::read_to_string(file_path)
        .unwrap_or_else(|_| panic!("Failed to read file {}", file_path));

      let module = parse_module(&file_content);
      assemble(&module)
    }

    RunFormat::Bytecode => {
      fs::read(file_path).unwrap_or_else(|_| panic!("Failed to read file {}", file_path))
    }
  })
}

fn show_help() {
  println!("vstc run");
  println!();
  println!("Run a ValueScript program");
  println!();
  println!("USAGE:");
  println!("    vstc run [OPTIONS] <file>");
  println!();
  println!("OPTIONS:");
  println!("    --assembly");
  println!("            Interpret <file> as assembly");
  println!();
  println!("    --bytecode");
  println!("            Interpret <file> as bytecode");
  println!();
  println!("    --typescript");
  println!("            Interpret <file> as typescript");
  println!();
  println!("NOTE:");
  println!("    <file> will be interpreted based on file extension if not otherwise specified");
}
