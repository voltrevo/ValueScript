use valuescript_compiler::{assemble, parse_module};

use crate::exit_command_failed::exit_command_failed;

pub fn assemble_command(args: &[String]) {
  if args.len() != 3 {
    exit_command_failed(args, None, "vstc assemble --help");
  }

  if args[2] == "-h" || args[2] == "--help" {
    show_help();
    return;
  }

  let read_result = std::fs::read_to_string(&args[2]);

  if read_result.is_err() {
    println!("Failed to read file {}", args[2]);
    return;
  }

  let content = read_result.expect("");
  let output_filename = "out.vsb";

  let module = parse_module(&content);
  let bytecode = assemble(&module);

  let write_result = std::fs::write(output_filename, &*bytecode);

  if write_result.is_err() {
    println!("Failed to write file {}", output_filename);
    std::process::exit(1);
  }
}

fn show_help() {
  println!("vstc assemble");
  println!();
  println!("Convert ValueScript assembly to bytecode");
  println!();
  println!("USAGE:");
  println!("  vstc assemble <file>");
}
