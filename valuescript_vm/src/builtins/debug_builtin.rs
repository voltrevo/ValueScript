use std::fmt;
use std::rc::Rc;

use crate::native_function::{NativeFunction, ThisWrapper};
use crate::vs_class::VsClass;
use crate::vs_value::{LoadFunctionResult, Val};

use super::builtin_object::BuiltinObject;

pub struct DebugBuiltin {}

pub static DEBUG_BUILTIN: DebugBuiltin = DebugBuiltin {};

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

static LOG: NativeFunction = NativeFunction {
  fn_: |_this: ThisWrapper, params: Vec<Val>| -> Result<Val, Val> {
    for p in params {
      println!("Debug.log: {}", p.pretty());
    }

    Ok(Val::Undefined)
  },
};
