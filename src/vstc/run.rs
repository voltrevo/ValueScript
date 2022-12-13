use std::rc::Rc;
use std::process::exit;
use std::path::Path;
use std::ffi::OsStr;

use super::assemble::assemble;
use super::compile::full_compile_raw;
use super::compile::parse;
use super::compile::compile;
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
    },
    _ => format_from_path(&args[argpos]),
  };

  let file_path = &args[argpos];
  argpos += 1;

  let bytecode = to_bytecode(format, file_path);

  let mut vm = VirtualMachine::new();
  let result = vm.run(&bytecode, &args[argpos..]);

  println!("{}", result);
}

pub fn full_run_raw(source: &str) -> String {
  let vsm = full_compile_raw(source);
  let bytecode = assemble(vsm.as_str());

  let mut vm = VirtualMachine::new();
  let result = vm.run(&bytecode, &[]);

  return format!("{}", result);
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
  }
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
      let ast = parse(file_path);
      let assembly_lines = compile(&ast);

      let mut assembly = String::new();

      for line in assembly_lines {
        assembly.push_str(&line);
        assembly.push('\n');
      }

      return assemble(&assembly);
    },

    RunFormat::Assembly => {
      let file_content = std::fs::read_to_string(&file_path)
        .expect(&std::format!("Failed to read file {}", file_path));

      assemble(&file_content)
    },

    RunFormat::Bytecode => Rc::new(
      std::fs::read(&file_path)
        .expect(&std::format!("Failed to read file {}", file_path))
    ),
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
