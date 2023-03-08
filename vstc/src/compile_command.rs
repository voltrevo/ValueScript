use std::fs::File;
use std::io::Write;
use std::process::exit;

use super::handle_diagnostics_cli::handle_diagnostics_cli;
use valuescript_compiler::gather_modules;
use valuescript_compiler::link_module;

pub fn compile_command(args: &Vec<String>) {
  if args.len() != 3 {
    println!("ERROR: Unrecognized command\n");
    show_help();
    exit(1);
  }

  let entry_path = &args[2];

  let abs_entry_path = std::fs::canonicalize(entry_path)
    .expect("Failed to get absolute path")
    .to_str()
    .expect("Failed to convert to str")
    .to_string();

  let gm = gather_modules(abs_entry_path, |path| {
    std::fs::read_to_string(path).map_err(|err| err.to_string())
  });

  for (path, diagnostics) in gm.diagnostics.iter() {
    handle_diagnostics_cli(path, diagnostics);
  }

  let link_module_result = link_module(&gm.entry_point, &gm.modules);

  // FIXME: Diagnostics from link_module should have paths associated
  handle_diagnostics_cli(&gm.entry_point, &link_module_result.diagnostics);

  let module = link_module_result
    .module
    .expect("Should have exited if module is None");

  let mut file = File::create("out.vsm").expect("Couldn't create out.vsm");

  file
    .write(module.to_string().as_bytes())
    .expect("Failed to write out.vsm");
}

fn show_help() {
  println!("vstc compile");
  println!();
  println!("Compile ValueScript");
  println!();
  println!("USAGE:");
  println!("    vstc compile <entry point>");
}
