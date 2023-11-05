use std::process::exit;
use std::rc::Rc;

use valuescript_vm::VirtualMachine;
use valuescript_vm::{vs_value::Val, DecoderMaker};

use crate::exit_command_failed::exit_command_failed;
use crate::to_bytecode::{format_from_path, to_bytecode, RunFormat};

pub fn run_command(args: &Vec<String>) {
  if args.len() < 3 {
    exit_command_failed(args, None, "vstc run --help");
  }

  let mut argpos = 2;

  if args[argpos] == "-h" || args[argpos] == "--help" {
    show_help();
    return;
  }

  let format = match args[argpos].chars().next() {
    Some('-') => {
      let res = format_from_option(&args[argpos]);
      argpos += 1;
      res
    }
    _ => format_from_path(&args[argpos]),
  };

  let file_path = &args[argpos];
  argpos += 1;

  let bytecode = Rc::new(to_bytecode(format, file_path));

  let mut vm = VirtualMachine::default();

  let val_args: Vec<Val> = args[argpos..]
    .iter()
    .map(|a| Val::String(Rc::from(a.clone())))
    .collect();

  match vm.run(
    None,
    &mut Val::Undefined,
    bytecode.decoder(0).decode_val(&mut vec![]),
    val_args,
  ) {
    Ok(Val::Undefined) => {}
    Ok(result) => {
      println!("{}", result.pretty());
    }
    Err(err) => {
      println!("Uncaught exception: {}", err.pretty());
      exit(1);
    }
  }
}

fn format_from_option(option: &String) -> RunFormat {
  return match option.as_str() {
    "--typescript" => RunFormat::TypeScript,
    "--assembly" => RunFormat::Assembly,
    "--bytecode" => RunFormat::Bytecode,
    _ => std::panic!("Unrecognized option {}", option),
  };
}

fn show_help() {
  println!("vstc run");
  println!();
  println!("Run a ValueScript program");
  println!();
  println!("USAGE:");
  println!("  vstc run [OPTIONS] <file>");
  println!();
  println!("OPTIONS:");
  println!("  --assembly");
  println!("    Interpret <file> as assembly");
  println!();
  println!("  --bytecode");
  println!("    Interpret <file> as bytecode");
  println!();
  println!("  --typescript");
  println!("    Interpret <file> as typescript");
  println!();
  println!("NOTE:");
  println!("  <file> will be interpreted based on file extension if not otherwise specified");
}
