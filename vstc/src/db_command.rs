use std::process::exit;

pub fn db_command(args: &[String]) {
  let mut help_case = false;

  match args.get(2).map(|s| s.as_str()) {
    Some("help") | Some("-h") | Some("--help") => help_case = true,
    _ => {}
  };

  match args.get(3).map(|s| s.as_str()) {
    Some("help") | Some("-h") | Some("--help") => help_case = true,
    _ => {}
  };

  if help_case {
    show_help();
    return;
  }

  let path = match args.get(2) {
    Some(path) => path.clone(),
    None => {
      println!("ERROR: Missing db path\n");
      show_help();
      exit(1);
    }
  };

  match args.get(3).map(|s| s.as_str()) {
    Some("new") => println!("TODO: create database {} using {:?}", path, args.get(4)),
    Some("call") => println!("TODO: on database {}, call {:?}", path, args.get(4)),
    Some("-i") => println!("TODO: use database {} interactively", path),
    arg => {
      if let Some(arg) = arg {
        if arg.starts_with("this.") {
          println!("TODO: on database {}, run {}", path, arg);
          return;
        }
      }

      println!("ERROR: Unrecognized db command {:?}\n", args);
      show_help();
      exit(1);
    }
  }
}

fn show_help() {
  println!("Usage: vstc db [DB_PATH] [COMMAND] [ARGS]");
  println!();
  println!("Commands:");
  println!("  help, -h, --help          Show this message");
  println!("  new [CLASS_FILE] [ARGS]   Create a new database");
  println!("  call [FN_FILE] [ARGS]     Call a function on the database");
  println!("  'this.[CODE]'             Run inline code within the database context");
  println!("  -i                        Enter interactive mode");
  println!();
  println!("Examples:");
  println!("  vstc db path/widget.vsdb new Widget.ts       Create a new widget database");
  println!("  vstc db path/widget.vsdb call useWidget.ts   Call useWidget.ts on the widget");
  println!("  vstc db path/widget.vsdb 'this.info()'       Call info method");
  println!("  vstc db path/widget.vsdb -i                  Enter interactive mode");
}
