use std::process::exit;

pub fn exit_command_failed(args: &[String], context: Option<&str>, help: &str) -> ! {
  println!("Command failed: {:?}", args);

  if let Some(context) = context {
    println!("  {}", context);
  }

  println!("  For help: {}", help);

  exit(1);
}
