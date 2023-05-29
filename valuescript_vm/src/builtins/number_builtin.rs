use std::fmt;
use std::rc::Rc;

use crate::native_function::{native_fn, ThisWrapper};
use crate::vs_value::ToVal;
use crate::{
  native_function::NativeFunction,
  vs_class::VsClass,
  vs_value::{LoadFunctionResult, Val},
  ValTrait,
};

use super::builtin_object::BuiltinObject;

pub struct NumberBuiltin {}

impl BuiltinObject for NumberBuiltin {
  fn bo_name() -> &'static str {
    "Number"
  }

  fn bo_sub(key: &str) -> Val {
    match key {
      "EPSILON" => core::f64::EPSILON.to_val(),
      "MAX_VALUE" => core::f64::MAX.to_val(),
      "MAX_SAFE_INTEGER" => (2f64.powi(53) - 1f64).to_val(),
      "MIN_SAFE_INTEGER" => (-(2f64.powi(53) - 1f64)).to_val(),
      "MIN_VALUE" => core::f64::MIN_POSITIVE.to_val(),
      "NEGATIVE_INFINITY" => core::f64::NEG_INFINITY.to_val(),
      "POSITIVE_INFINITY" => core::f64::INFINITY.to_val(),
      "NaN" => core::f64::NAN.to_val(),
      "isFinite" => IS_FINITE.to_val(),
      "isInteger" => IS_INTEGER.to_val(),
      "isNaN" => IS_NAN.to_val(),
      "isSafeInteger" => IS_SAFE_INTEGER.to_val(),
      "parseFloat" => PARSE_FLOAT.to_val(),
      "parseInt" => PARSE_INT.to_val(),
      _ => Val::Undefined,
    }
  }

  fn bo_load_function() -> LoadFunctionResult {
    LoadFunctionResult::NativeFunction(|_: ThisWrapper, params: Vec<Val>| -> Result<Val, Val> {
      Ok(if let Some(value) = params.get(0) {
        Val::Number(value.to_number())
      } else {
        Val::Number(0.0)
      })
    })
  }

  fn bo_as_class_data() -> Option<Rc<VsClass>> {
    None
  }
}

impl fmt::Display for NumberBuiltin {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "function Number() {{ [native code] }}")
  }
}

pub static IS_FINITE: NativeFunction = native_fn(|_this, params| {
  Ok(if let Some(value) = params.get(0) {
    let number = value.to_number();
    Val::Bool(number.is_finite())
  } else {
    Val::Bool(false)
  })
});

static IS_INTEGER: NativeFunction = native_fn(|_this, params| {
  let num = match params.get(0) {
    Some(n) => n.to_number(),
    None => return Ok(Val::Bool(false)),
  };

  let is_finite = num.is_finite();
  let is_integer = num.floor() == num;

  Ok(Val::Bool(is_finite && is_integer))
});

pub static IS_NAN: NativeFunction = native_fn(|_this, params| {
  Ok(if let Some(value) = params.get(0) {
    let number = value.to_number();
    Val::Bool(number.is_nan())
  } else {
    Val::Bool(false)
  })
});

static IS_SAFE_INTEGER: NativeFunction = native_fn(|_this, params| {
  let num = match params.get(0) {
    Some(n) => n.to_number(),
    None => return Ok(Val::Bool(false)),
  };

  let is_finite = num.is_finite();
  let is_integer = num.floor() == num;
  let min_safe_integer = -(2f64.powi(53) - 1f64);
  let max_safe_integer = 2f64.powi(53) - 1f64;
  let in_safe_range = min_safe_integer <= num && num <= max_safe_integer;

  Ok(Val::Bool(is_finite && is_integer && in_safe_range))
});

pub static PARSE_FLOAT: NativeFunction = native_fn(|_this, params| {
  Ok(if let Some(value) = params.get(0) {
    let string_value = value.to_string().trim().to_string();

    match string_value.parse::<f64>() {
      Ok(number) => Val::Number(number),
      Err(_) => Val::Number(core::f64::NAN),
    }
  } else {
    Val::Number(core::f64::NAN)
  })
});

pub static PARSE_INT: NativeFunction = native_fn(|_this, params| {
  Ok(if let Some(value) = params.get(0) {
    let string_value = value.to_string().trim_start().to_string();
    let radix = params.get(1).and_then(|v| v.to_index()).unwrap_or(10);

    if radix < 2 || radix > 36 {
      return Ok(Val::Number(core::f64::NAN));
    }

    let (is_negative, string_value) = if string_value.starts_with('-') {
      (true, &string_value[1..])
    } else {
      (false, string_value.as_str())
    };

    let string_value = match string_value.find(|c: char| !c.is_digit(radix as u32)) {
      Some(pos) => &string_value[..pos],
      None => &string_value,
    };

    match i64::from_str_radix(string_value, radix as u32) {
      Ok(number) => {
        let number = if is_negative { -number } else { number };
        Val::Number(number as f64)
      }
      Err(_) => Val::Number(core::f64::NAN),
    }
  } else {
    Val::Number(core::f64::NAN)
  })
});
