mod assemble_command;
mod compile_command;
mod handle_diagnostics_cli;
mod resolve_entry_path;
mod run_command;
mod test_inputs;

use std::env;
use std::process::exit;

use assemble_command::assemble_command;
use compile_command::compile_command;
use run_command::run_command;

fn main() {
  let args: Vec<String> = env::args().collect();

  match args.get(1).map(|s| s.as_str()) {
    Some("help") | Some("-h") | Some("--help") | None => show_help(),
    Some("assemble") => assemble_command(&args),
    Some("run") => run_command(&args),
    Some("compile") => compile_command(&args),
    _ => {
      println!("ERROR: Unrecognized command\n");
      show_help();
      exit(1);
    }
  }
}

fn show_help() {
  println!("ValueScript toolchain 0.1.0");
  println!();
  println!("USAGE:");
  println!("    vstc [OPTIONS] [SUBCOMMAND]");
  println!();
  println!("OPTIONS:");
  println!("    -h, --help");
  println!("            Print help information");
  println!();
  println!("    -V, --version");
  println!("            Print version information");
  println!();
  println!("SUBCOMMANDS:");
  println!("    compile");
  println!("            Compile an entry point");
  println!();
  println!("    assemble");
  println!("            Convert assembly to bytecode");
  println!();
  println!("    disassemble");
  println!("            Convert bytecode to assembly");
  println!();
  println!("    run");
  println!("            Run a program");
  println!();
  println!("    repl");
  println!("            Read Eval Print Loop");
  println!();
  println!("    host");
  println!("            Start database server");
}
