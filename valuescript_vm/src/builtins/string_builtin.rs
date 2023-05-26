use std::fmt;
use std::rc::Rc;

use num_bigint::BigInt;

use crate::native_function::ThisWrapper;
use crate::vs_value::ToVal;
use crate::{builtins::range_error_builtin::to_range_error, range_error};
use crate::{
  native_function::NativeFunction,
  vs_array::VsArray,
  vs_class::VsClass,
  vs_object::VsObject,
  vs_value::{LoadFunctionResult, Val, VsType},
  ValTrait,
};

use super::type_error_builtin::ToTypeError;

pub struct StringBuiltin {}

pub static STRING_BUILTIN: StringBuiltin = StringBuiltin {};

impl ValTrait for StringBuiltin {
  fn typeof_(&self) -> VsType {
    VsType::Object
  }
  fn to_number(&self) -> f64 {
    core::f64::NAN
  }
  fn to_index(&self) -> Option<usize> {
    None
  }
  fn is_primitive(&self) -> bool {
    false
  }
  fn is_truthy(&self) -> bool {
    true
  }
  fn is_nullish(&self) -> bool {
    false
  }
  fn bind(&self, _params: Vec<Val>) -> Option<Val> {
    None
  }
  fn as_bigint_data(&self) -> Option<BigInt> {
    None
  }
  fn as_array_data(&self) -> Option<Rc<VsArray>> {
    None
  }
  fn as_object_data(&self) -> Option<Rc<VsObject>> {
    None
  }
  fn as_class_data(&self) -> Option<Rc<VsClass>> {
    None
  }

  fn load_function(&self) -> LoadFunctionResult {
    LoadFunctionResult::NativeFunction(to_string)
  }

  fn sub(&self, key: Val) -> Result<Val, Val> {
    // Not supported: fromCharCode.
    // See charAt etc in string_methods.rs.

    Ok(match key.to_string().as_str() {
      "fromCodePoint" => Val::Static(&FROM_CODE_POINT),
      // "fromCharCode" => Val::Static(&FROM_CHAR_CODE),
      // "raw" => Val::Static(&RAW),                     // TODO
      _ => Val::Undefined,
    })
  }

  fn submov(&mut self, _key: Val, _value: Val) -> Result<(), Val> {
    Err("Cannot assign to subscript of String builtin".to_type_error())
  }

  fn next(&mut self) -> LoadFunctionResult {
    LoadFunctionResult::NotAFunction
  }

  fn pretty_fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "\x1b[36m[String]\x1b[39m")
  }

  fn codify(&self) -> String {
    "String".into()
  }
}

impl fmt::Display for StringBuiltin {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "function String() {{ [native code] }}")
  }
}

static FROM_CODE_POINT: NativeFunction = NativeFunction {
  fn_: |_this: ThisWrapper, params: Vec<Val>| -> Result<Val, Val> {
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
  },
};

fn to_string(_: ThisWrapper, params: Vec<Val>) -> Result<Val, Val> {
  Ok(if let Some(value) = params.get(0) {
    value.clone().to_val_string()
  } else {
    "".to_val()
  })
}
