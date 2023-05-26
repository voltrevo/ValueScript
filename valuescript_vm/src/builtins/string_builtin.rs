use std::fmt;
use std::rc::Rc;

use crate::native_function::{native_fn, ThisWrapper};
use crate::vs_value::ToVal;
use crate::{builtins::range_error_builtin::to_range_error, range_error};
use crate::{
  native_function::NativeFunction,
  vs_class::VsClass,
  vs_value::{LoadFunctionResult, Val},
  ValTrait,
};

use super::builtin_object::BuiltinObject;

pub struct StringBuiltin {}

pub static STRING_BUILTIN: StringBuiltin = StringBuiltin {};

impl BuiltinObject for StringBuiltin {
  fn bo_name() -> &'static str {
    "String"
  }

  fn bo_sub(key: &str) -> Val {
    // Not supported: fromCharCode.
    // See charAt etc in string_methods.rs.

    match key {
      "fromCodePoint" => Val::Static(&FROM_CODE_POINT),
      // "fromCharCode" => Val::Static(&FROM_CHAR_CODE),
      // "raw" => Val::Static(&RAW),                     // TODO
      _ => Val::Undefined,
    }
  }

  fn bo_load_function() -> LoadFunctionResult {
    LoadFunctionResult::NativeFunction(to_string)
  }

  fn bo_as_class_data() -> Option<Rc<VsClass>> {
    None
  }
}

impl fmt::Display for StringBuiltin {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "function String() {{ [native code] }}")
  }
}

static FROM_CODE_POINT: NativeFunction = native_fn(|_this, params| {
  let mut result = String::new();

  for param in params {
    let code_point = param.to_number() as u32; // TODO: Check overflow behavior

    let char = match std::char::from_u32(code_point) {
      Some(c) => c,
      None => return range_error!("Invalid code point"),
    };

    result.push(char);
  }

  Ok(result.to_val())
});

fn to_string(_: ThisWrapper, params: Vec<Val>) -> Result<Val, Val> {
  Ok(if let Some(value) = params.get(0) {
    value.clone().to_val_string()
  } else {
    "".to_val()
  })
}
