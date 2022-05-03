use std::rc::Rc;
use std::process::exit;
use std::path::Path;
use std::ffi::OsStr;

use super::assemble::assemble;
use super::virtual_machine::VirtualMachine;

pub fn command(args: &Vec<String>) {
  if args.len() < 3 {
    println!("ERROR: Unrecognized command\n");
    show_help();
    exit(1);
  }

  if args[2] == "-h" || args[2] == "--help" {
    show_help();
    return;
  }

  let format = match args.len() {
    3 => format_from_path(&args[2]),
    4 => format_from_option(&args[2]),
    _ => {
      println!("ERROR: Unrecognized command\n");
      show_help();
      exit(1);
    }
  };

  let file_path = match args.len() {
    3 => &args[2],
    4 => &args[3],
    _ => std::panic!("Should not be possible"),
  };

  let bytecode = to_bytecode(format, &file_path);

  let mut vm = VirtualMachine::new();
  let result = vm.run(&bytecode);

  println!("{}", result);
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
      let _file_content = Rc::new(
        std::fs::read(&file_path)
          .expect(&std::format!("Failed to read file {}", file_path))
      );
  
      std::panic!("Not implemented: convert typescript to bytecode");
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
