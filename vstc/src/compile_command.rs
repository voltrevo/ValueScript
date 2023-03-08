use std::fs::File;
use std::io::Write;
use std::process::exit;

use super::handle_diagnostics_cli::handle_diagnostics_cli;
use valuescript_compiler::compile;

pub fn compile_command(args: &Vec<String>) {
  if args.len() != 3 {
    println!("ERROR: Unrecognized command\n");
    show_help();
    exit(1);
  }

  let source = std::fs::read_to_string(&args[2]).expect("Failed to read file");
  let compiler_output = compile(&source);

  handle_diagnostics_cli(&args[2], &compiler_output.diagnostics);

  let mut file = File::create("out.vsm").expect("Couldn't create out.vsm");

  for line in compiler_output.module.as_lines() {
    file
      .write_all(line.as_bytes())
      .expect("Failed to write line");
    file.write_all(b"\n").expect("Failed to write line");
  }
}

fn show_help() {
  println!("vstc compile");
  println!();
  println!("Compile ValueScript");
  println!();
  println!("USAGE:");
  println!("    vstc compile <entry point>");
}
