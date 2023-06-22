use std::fmt;
use std::rc::Rc;

use num_bigint::BigInt;

use crate::native_function::ThisWrapper;
use crate::{
  vs_class::VsClass,
  vs_value::{LoadFunctionResult, Val},
};

use super::builtin_object::BuiltinObject;
use super::error_builtin::ToError;
use super::range_error_builtin::ToRangeError;
use super::type_error_builtin::ToTypeError;

pub struct BigIntBuiltin {}

impl BuiltinObject for BigIntBuiltin {
  fn bo_name() -> &'static str {
    "BigInt"
  }

  fn bo_sub(_key: &str) -> Val {
    Val::Undefined
  }

  fn bo_load_function() -> LoadFunctionResult {
    LoadFunctionResult::NativeFunction(|_: ThisWrapper, params: Vec<Val>| -> Result<Val, Val> {
      match params.get(0) {
        Some(Val::Number(value)) => {
          if *value != f64::floor(*value) {
            Err(
              format!(
                "{} can't be converted to BigInt because it isn't an integer",
                value
              )
              .to_range_error(),
            )
          } else {
            Ok(Val::BigInt(BigInt::from(*value as u64)))
          }
        }
        Some(Val::Undefined) | None => Err("Can't convert undefined to BigInt".to_type_error()),
        _ => Err("TODO: Other BigInt conversions".to_error()),
      }
    })
  }

  fn bo_as_class_data() -> Option<Rc<VsClass>> {
    None
  }
}

impl fmt::Display for BigIntBuiltin {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "function BigInt() {{ [native code] }}")
  }
}
