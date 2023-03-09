use std::fs::File;
use std::io::Write;
use std::process::exit;

use super::handle_diagnostics_cli::handle_diagnostics_cli;
use valuescript_compiler::gather_modules;
use valuescript_compiler::link_module;
use valuescript_compiler::resolve_path;
use valuescript_compiler::ResolvedPath;

pub fn compile_command(args: &Vec<String>) {
  if args.len() != 3 {
    println!("ERROR: Unrecognized command\n");
    show_help();
    exit(1);
  }

  let entry_path = &args[2];

  // Like cwd (current working dir), but it's cwd/file.
  // This is a bit of a hack so we can use resolve_path to get the absolute path of the entry point.
  let cwd_file = ResolvedPath {
    path: std::env::current_dir()
      .expect("Failed to get current directory")
      .as_path()
      .join("file")
      .to_str()
      .expect("Failed to convert to str")
      .to_string(),
  };

  let resolved_entry_path = resolve_path(&cwd_file, entry_path);

  let gm = gather_modules(resolved_entry_path, |path| {
    std::fs::read_to_string(path).map_err(|err| err.to_string())
  });

  for (path, diagnostics) in gm.diagnostics.iter() {
    handle_diagnostics_cli(&path.path, diagnostics);
  }

  let link_module_result = link_module(&gm.entry_point, &gm.modules);

  // FIXME: Diagnostics from link_module should have paths associated
  handle_diagnostics_cli(&gm.entry_point.path, &link_module_result.diagnostics);

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
