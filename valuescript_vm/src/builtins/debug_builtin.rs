use std::fmt;
use std::rc::Rc;

use crate::native_function::{native_fn, NativeFunction};
use crate::vs_class::VsClass;
use crate::vs_value::{LoadFunctionResult, Val};

use super::builtin_object::BuiltinObject;

pub struct DebugBuiltin {}

impl BuiltinObject for DebugBuiltin {
  fn bo_name() -> &'static str {
    "Debug"
  }

  fn bo_sub(key: &str) -> Val {
    Val::Static(match key {
      "log" => &LOG,
      _ => return Val::Undefined,
    })
  }

  fn bo_load_function() -> LoadFunctionResult {
    LoadFunctionResult::NotAFunction
  }

  fn bo_as_class_data() -> Option<Rc<VsClass>> {
    None
  }
}

impl fmt::Display for DebugBuiltin {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "[object Debug]")
  }
}

static LOG: NativeFunction = native_fn(|_this, params| {
  print!("Debug.log:");

  for p in params {
    print!(" {}", p.pretty());
  }

  println!();

  Ok(Val::Undefined)
});
