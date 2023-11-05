// use storage::{SledBackend, Storage};

pub fn console_command(args: &[String]) {
  // let mut storage = Storage::new(SledBackend::open("./state.vsdb").unwrap());

  match args.get(0).map(String::as_str) {
    Some("help") => show_help(),
    Some("new") => todo!(),
    Some("start") => todo!(),
    _ => todo!(),
  }
}

fn show_help() {
  println!("vstc console");
  println!();
  println!("ValueScript console applications");
  println!();
  println!("USAGE:");
  println!("  vstc console [COMMAND] [ARGS]");
  println!();
  println!("Commands:");
  println!("  help, -h, --help          Show this message");
  println!("  new [CLASS_FILE] [ARGS]   Create a new console app");
  println!("  start [APP_FILE] [ARGS]   Start an existing console app");
  println!();
  println!("Examples:");
  println!("  vstc console new Tetris.ts       Create a tetris game");
  println!("  vstc console start tetris.vsdb   Start tetris");
}
