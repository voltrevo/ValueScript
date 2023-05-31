use std::fs::File;
use std::io::Write;
use std::process::exit;

use crate::resolve_entry_path::resolve_entry_path;

use super::handle_diagnostics_cli::handle_diagnostics_cli;
use valuescript_compiler::compile;

pub fn compile_command(args: &Vec<String>) {
  if args.len() != 3 {
    println!("ERROR: Unrecognized command\n");
    show_help();
    exit(1);
  }

  let resolved_entry_path = resolve_entry_path(&args[2]);

  let compile_result = compile(resolved_entry_path, |path| {
    std::fs::read_to_string(path).map_err(|err| err.to_string())
  });

  for (path, diagnostics) in compile_result.diagnostics.iter() {
    handle_diagnostics_cli(&path.path, diagnostics);
  }

  let module = compile_result
    .module
    .expect("Should have exited if module is None");

  let mut file = File::create("out.vsm").expect("Couldn't create out.vsm");

  file
    .write(module.to_string().as_bytes())
    .expect("Failed to write out.vsm");

  file.write(b"\n").expect("Failed to write out.vsm");
}

fn show_help() {
  println!("vstc compile");
  println!();
  println!("Compile ValueScript");
  println!();
  println!("USAGE:");
  println!("    vstc compile <entry point>");
}
