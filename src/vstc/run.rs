use super::assemble::assemble;
use super::virtual_machine::VirtualMachine;
use std::process::exit;

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

  if args.len() == 3 {
    std::panic!("Not implemented: Run script");
  }

  let bytecode = to_bytecode(&args[2], &args[3]);

  let mut vm = VirtualMachine::new();
  vm.load(bytecode);
  vm.run();
  vm.print();
}

fn to_bytecode(option: &String, file_path: &String) -> Vec<u8> {
  if option == "--assembly" {
    let file_content = std::fs::read_to_string(&file_path)
      .expect(&std::format!("Failed to read file {}", file_path));

    return assemble(&file_content);
  }

  if option == "--bytecode" {
    return std::fs::read(&file_path)
      .expect(&std::format!("Failed to read file {}", file_path));
  }

  std::panic!("Unrecognized option {}", option);
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
}
