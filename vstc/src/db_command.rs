use std::{io::Write, process::exit, rc::Rc};

use storage::{storage_head_ptr, SledBackend, Storage, StorageReader};
use valuescript_compiler::{assemble, compile_str};
use valuescript_vm::{
  vs_value::{ToVal, Val},
  Bytecode, DecoderMaker, VirtualMachine,
};

use crate::{
  create_db::create_db,
  exit_command_failed::exit_command_failed,
  handle_diagnostics_cli::handle_diagnostics_cli,
  parse_command_line::parse_command_line,
  to_bytecode::{format_from_path, to_bytecode},
};

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
      exit_command_failed(args, Some("Missing db path"), "vstc db help");
    }
  };

  let mut storage = Storage::new(SledBackend::open(path).unwrap());

  match args.get(3).map(|s| s.as_str()) {
    Some("new") => db_new(&mut storage, args.get(4..).unwrap_or_default()),
    Some("call") => db_call(&mut storage, args.get(4..).unwrap_or_default()),
    Some("-i") => db_interactive(&mut storage),
    arg => 'b: {
      if let Some(arg) = arg {
        if arg.starts_with('{') || arg.starts_with('(') {
          break 'b db_run_inline(&mut storage, arg);
        }
      }

      exit_command_failed(args, None, "vstc db help");
    }
  }
}

fn show_help() {
  println!("vstc db");
  println!();
  println!("ValueScript database functionality");
  println!();
  println!("USAGE:");
  println!("  vstc db [DB_PATH] [COMMAND] [ARGS]");
  println!();
  println!("Commands:");
  println!("  help, -h, --help          Show this message");
  println!("  new [CLASS_FILE] [ARGS]   Create a new database");
  println!("  call [FN_FILE] [ARGS]     Call a function on the database");
  println!("  '([EXPRESSION])'          Run expression with database as `this`");
  println!("  '{{[FN BODY]}}'             Run code block with database as `this`");
  println!("  -i                        Enter interactive mode");
  println!();
  println!("Examples:");
  println!("  vstc db path/widget.vsdb new Widget.ts       Create a new widget database");
  println!("  vstc db path/widget.vsdb call useWidget.ts   Call useWidget.ts on the widget");
  println!("  vstc db path/widget.vsdb '(this.info())'     Call info method");
  println!("  vstc db path/widget.vsdb '{{ const t = this; return t.info(); }}'");
  println!("                                               Call info method (enforcing read-only)");
  println!("  vstc db path/widget.vsdb -i                  Enter interactive mode");
}

fn db_new(storage: &mut Storage<SledBackend>, args: &[String]) {
  let class_path = match args.get(0) {
    Some(class_path) => class_path,
    None => {
      exit_command_failed(args, Some("Missing class file"), "vstc db help");
    }
  };

  let args = args
    .get(1..)
    .unwrap_or_default()
    .iter()
    .map(|s| s.clone().to_val())
    .collect::<Vec<_>>();

  create_db(storage, class_path, &args).expect("Failed to write to db");

  println!("Created database");
}

fn db_call(storage: &mut Storage<SledBackend>, args: &[String]) {
  let fn_file = match args.get(0) {
    Some(fn_file) => fn_file,
    None => exit_command_failed(args, Some("Missing function file"), "vstc db help"),
  };

  let fn_ = Rc::new(to_bytecode(format_from_path(fn_file), fn_file))
    .decoder(0)
    .decode_val(&mut vec![]);

  let args = args
    .get(1..)
    .unwrap_or_default()
    .iter()
    .map(|s| s.clone().to_val())
    .collect::<Vec<_>>();

  let mut vm = VirtualMachine::default();

  let mut instance = storage
    .get_head(storage_head_ptr(b"state"))
    .unwrap()
    .unwrap();

  match vm.run(None, &mut instance, fn_, args) {
    Ok(res) => {
      println!("{}", res.pretty());
    }
    Err(err) => {
      println!("Uncaught exception: {}", err.pretty());
      exit(1);
    }
  }

  storage
    .set_head(storage_head_ptr(b"state"), &instance)
    .unwrap();
}

fn db_run_inline(storage: &mut Storage<SledBackend>, source: &str) {
  let mut vm = VirtualMachine::default();

  let mut instance = storage
    .get_head::<Val>(storage_head_ptr(b"state"))
    .unwrap()
    .unwrap();

  let full_source = {
    if source.starts_with('(') {
      format!("export default function() {{ return (\n  {}\n); }}", source)
    } else if source.starts_with('{') {
      format!("export default function() {}", source)
    } else {
      panic!("Unrecognized inline code: {}", source);
    }
  };

  let compile_result = compile_str(&full_source);

  for (path, diagnostics) in compile_result.diagnostics.iter() {
    // TODO: Fix exit call
    handle_diagnostics_cli(&path.path, diagnostics);
  }

  let bytecode = Rc::new(Bytecode::new(assemble(
    &compile_result
      .module
      .expect("Should have exited if module is None"),
  )));

  let fn_ = bytecode.decoder(0).decode_val(&mut vec![]);

  match vm.run(None, &mut instance, fn_, vec![]) {
    Ok(res) => {
      println!("{}", res.pretty());
    }
    Err(err) => {
      println!("Uncaught exception: {}", err.pretty());
      exit(1);
    }
  }

  storage
    .set_head(storage_head_ptr(b"state"), &instance)
    .unwrap();
}

fn db_interactive(storage: &mut Storage<SledBackend>) {
  loop {
    let mut input = String::new();

    print!("> ");
    std::io::stdout().flush().unwrap();
    std::io::stdin().read_line(&mut input).unwrap();
    input.pop();

    let args = parse_command_line(&input);

    match args.get(0).map(|s| s.as_str()) {
      Some("help") => {
        // TODO: help (it's a bit different - code isn't quoted (TODO: quoted should work too))
        println!("TODO: help");
      }
      Some("exit" | "quit") => break,
      Some("new") => db_new(storage, args.get(1..).unwrap_or_default()),
      Some("call") => db_call(storage, args.get(1..).unwrap_or_default()),
      _ => 'b: {
        if input.starts_with('{') || input.starts_with('(') {
          break 'b db_run_inline(storage, &input);
        }

        println!("Command failed: {:?}", args);
        println!("  For help: help");
      }
    }
  }
}
