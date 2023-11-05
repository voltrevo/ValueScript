// use storage::{SledBackend, Storage};

use std::{
  io::{stdin, stdout, Stdout, Write},
  path::{Path, PathBuf},
  process::exit,
  rc::Rc,
};

use storage::{storage_head_ptr, SledBackend, Storage, StorageReader};
use termion::{
  event::Key,
  input::{MouseTerminal, TermRead},
  raw::{IntoRawMode, RawTerminal},
  terminal_size,
};
use valuescript_compiler::{assemble, compile_str};
use valuescript_vm::{
  vs_object::VsObject,
  vs_value::{ToVal, Val},
  Bytecode, DecoderMaker, ValTrait, VirtualMachine,
};

use crate::{create_db::create_db, exit_command_failed::exit_command_failed};

pub fn console_command(args: &[String]) {
  match args.get(2).map(String::as_str) {
    Some("help") => show_help(),
    Some("new") => console_new(args),
    Some("start") => console_start(args),
    _ => exit_command_failed(args, None, "vstc console help"),
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

fn console_new(args: &[String]) {
  let class_path = match args.get(3) {
    Some(class_path) => class_path,
    None => exit_command_failed(args, Some("Missing class file"), "vstc console help"),
  };

  let db_path = make_db_path(class_path);

  let mut storage = Storage::new(SledBackend::open(db_path).expect("Failed to open db"));

  let args = args
    .get(1..)
    .unwrap_or_default()
    .iter()
    .map(|s| s.clone().to_val())
    .collect::<Vec<_>>();

  create_db(&mut storage, class_path, &args).expect("Failed to write to db");

  println!("Created database");
}

fn console_start(args: &[String]) {
  let db_path = match args.get(3) {
    Some(db_path) => db_path,
    None => exit_command_failed(args, Some("Missing db file"), "vstc console help"),
  };

  let storage = Storage::new(SledBackend::open(db_path).expect("Failed to open db"));

  let mut db = storage
    .get_head::<Val>(storage_head_ptr(b"state"))
    .expect("Read failed")
    .expect("no state head");

  let mut vm = VirtualMachine::default();

  let view = vm
    .run(
      None,
      &mut db,
      inline_valuescript(
        "export default function() {
          const db = this;
          return db.createView();
        }",
      ),
      vec![],
    )
    .or_exit_uncaught();

  let ctx = VsObject {
    string_map: [("db".to_string(), db), ("view".to_string(), view)]
      .iter()
      .cloned()
      .collect(),
    symbol_map: Default::default(),
    prototype: Val::Void,
  }
  .to_val();

  let mut app = ConsoleApp {
    ctx,
    stdout: MouseTerminal::from(stdout().into_raw_mode().unwrap()),
    storage,
  };

  app.run();
}

struct ConsoleApp {
  ctx: Val,
  stdout: MouseTerminal<RawTerminal<Stdout>>,
  storage: Storage<SledBackend>,
}

impl ConsoleApp {
  fn run(&mut self) {
    self.render();

    for c in stdin().events() {
      let evt = c.unwrap();

      match evt {
        termion::event::Event::Key(k) => {
          let key_str = match k {
            Key::Ctrl('c') | Key::Ctrl('d') => break,
            Key::Left => "ArrowLeft".to_string(),
            Key::Right => "ArrowRight".to_string(),
            Key::Up => "ArrowUp".to_string(),
            Key::Down => "ArrowDown".to_string(),
            Key::Char(c) => c.to_string(),
            _ => continue,
          };

          let on_key_down = self
            .ctx
            .sub(&"db".to_val())
            .or_exit_uncaught()
            .sub(&"onKeyDown".to_val())
            .or_exit_uncaught();

          let mut vm = VirtualMachine::default();

          vm.run(None, &mut self.ctx, on_key_down, vec![key_str.to_val()])
            .or_exit_uncaught();

          self.render();

          let db = self.ctx.sub(&"db".to_val()).or_exit_uncaught();

          self
            .storage
            .set_head(storage_head_ptr(b"state"), &db)
            .unwrap();
        }
        termion::event::Event::Mouse(_) => {}
        termion::event::Event::Unsupported(_) => todo!(),
      }
    }
  }

  fn render(&mut self) {
    let mut vm = VirtualMachine::default();

    let render = self
      .ctx
      .sub(&"db".to_val())
      .or_exit_uncaught()
      .sub(&"render".to_val())
      .or_exit_uncaught();

    let (width, height) = terminal_size().unwrap();

    let info = VsObject {
      string_map: [
        ("screenWidth".to_string(), (width as f64).to_val()),
        ("screenHeight".to_string(), (height as f64).to_val()),
      ]
      .iter()
      .cloned()
      .collect(),
      symbol_map: Default::default(),
      prototype: Val::Void,
    }
    .to_val();

    match vm
      .run(None, &mut self.ctx, render, vec![info])
      .or_exit_uncaught()
    {
      Val::Array(arr) => {
        write!(
          self.stdout,
          "{}{}",
          termion::clear::All,
          termion::cursor::Goto(1, 1),
        )
        .unwrap();

        for (i, line) in arr.elements.iter().enumerate() {
          match line {
            Val::String(s) => {
              write!(
                self.stdout,
                "{}{}",
                termion::cursor::Goto(1, (i as u16) + 1),
                s
              )
              .unwrap();
            }
            line => {
              println!("ERROR: Non-string line: {}", line.pretty());
              exit(1);
            }
          }
        }

        self.stdout.flush().unwrap();
      }
      Val::Undefined => exit(0),
      non_str => {
        println!("ERROR: Non-array render: {}", non_str.pretty());
        exit(1);
      }
    }
  }
}

fn make_db_path(class_path: &str) -> String {
  // Convert the class_path to a Path
  let path = Path::new(class_path);

  // Extract the stem (file name without extension) and convert the first letter to lowercase
  let file_stem = path
    .file_stem()
    .and_then(|s| s.to_str())
    .map(|s| {
      let mut c = s.chars();
      match c.next() {
        None => String::new(),
        Some(f) => f.to_lowercase().collect::<String>() + c.as_str(),
      }
    })
    .unwrap_or_default();

  // Create a new PathBuf from the directory of the original path
  let mut new_path = PathBuf::new();
  if let Some(parent) = path.parent() {
    new_path.push(parent);
  }

  // Set the new file name with the lowercase first letter and with the new extension
  new_path.push(file_stem);
  new_path.set_extension("vsdb");

  // Convert the PathBuf back to a String and return it
  new_path.to_str().unwrap_or_default().to_string()
}

fn inline_valuescript(source: &str) -> Val {
  Rc::new(Bytecode::new(assemble(
    &compile_str(source).module.unwrap(),
  )))
  .decoder(0)
  .decode_val(&mut vec![])
}

trait OrExitUncaught {
  fn or_exit_uncaught(&self) -> Val;
}

impl OrExitUncaught for Result<Val, Val> {
  fn or_exit_uncaught(&self) -> Val {
    match self {
      Ok(val) => val.clone(),
      Err(err) => {
        println!("Uncaught exception: {}", err.pretty());
        exit(1);
      }
    }
  }
}
