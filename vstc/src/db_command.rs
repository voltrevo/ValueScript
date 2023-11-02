use std::{process::exit, rc::Rc};

use storage::{storage_head_ptr, SledBackend, Storage};
use valuescript_compiler::asm;
use valuescript_vm::{
  vs_value::{ToVal, Val},
  DecoderMaker, VirtualMachine,
};

use crate::to_bytecode::{format_from_path, to_bytecode};

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
    Some("new") => db_new(&path, args.get(4..).unwrap_or_default()),
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
  println!("vstc db [DB_PATH] [COMMAND] [ARGS]");
  println!();
  println!("ValueScript database functionality");
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

fn db_new(path: &str, args: &[String]) {
  let class_file = match args.get(0) {
    Some(class_file) => class_file,
    None => {
      println!("ERROR: Missing class file\n");
      show_help();
      exit(1);
    }
  };

  let class = Rc::new(to_bytecode(format_from_path(class_file), class_file))
    .decoder(0)
    .decode_val(&mut vec![]);

  let args = args
    .get(1..)
    .unwrap_or_default()
    .iter()
    .map(|s| s.clone().to_val())
    .collect::<Vec<_>>()
    .to_val();

  // TODO: Use compile_str instead. Need to implement rest params: `new Class(...args)`.
  let create = asm::inline(
    "export @create {}

    @create = function (%class, %args) {
      new %class %args %return
    }",
  );

  let mut vm = VirtualMachine::default();

  let instance = match vm.run(None, &mut Val::Undefined, create, vec![class, args]) {
    Ok(instance) => instance,
    Err(err) => {
      println!("Uncaught exception: {}", err.pretty());
      exit(1);
    }
  };

  let mut storage = Storage::new(SledBackend::open(path).unwrap());

  storage
    .set_head(storage_head_ptr(b"state"), &instance)
    .unwrap();

  println!("Created database at {}", path);
}
