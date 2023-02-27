use std::process::exit;

use valuescript_compiler::assemble;

pub fn assemble_command(args: &Vec<String>) {
  if args.len() != 3 {
    println!("ERROR: Unrecognized command\n");
    show_help();
    exit(1);
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
  let bytecode = assemble(&content);

  let write_result = std::fs::write(output_filename, &*bytecode);

  if write_result.is_err() {
    println!("Failed to write file {}", output_filename);
    std::process::exit(1);
  }
}

fn show_help() {
  println!("vstc assemble");
  println!("");
  println!("Convert ValueScript assembly to bytecode");
  println!("");
  println!("USAGE:");
  println!("    vstc assemble <file>");
}
