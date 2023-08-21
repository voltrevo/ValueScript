use std::fmt;
use std::rc::Rc;

use crate::native_function::{native_fn, NativeFunction};
use crate::vs_class::VsClass;
use crate::vs_value::{LoadFunctionResult, ToVal, Val};

use super::builtin_object::BuiltinObject;

pub struct ConsoleBuiltin {}

impl BuiltinObject for ConsoleBuiltin {
  fn bo_name() -> &'static str {
    "console"
  }

  fn bo_sub(key: &str) -> Val {
    match key {
      "log" => LOG.to_val(),

      _ => Val::Undefined,
    }
  }

  fn bo_load_function() -> LoadFunctionResult {
    LoadFunctionResult::NotAFunction
  }

  fn bo_as_class_data() -> Option<Rc<VsClass>> {
    None
  }
}

impl fmt::Display for ConsoleBuiltin {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "[object console]")
  }
}

static LOG: NativeFunction = native_fn(|_this, params| {
  for (i, p) in params.iter().enumerate() {
    if i > 0 {
      print!(" ");
    }

    print!("{}", p);
  }

  println!();

  Ok(Val::Undefined)
});
