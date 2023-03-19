use std::rc::Rc;

use crate::{
  native_function::NativeFunction,
  vs_array::VsArray,
  vs_class::VsClass,
  vs_object::VsObject,
  vs_value::{LoadFunctionResult, Val, VsType},
  ValTrait,
};

pub struct NumberBuiltin {}

pub static NUMBER_BUILTIN: NumberBuiltin = NumberBuiltin {};

impl ValTrait for NumberBuiltin {
  fn typeof_(&self) -> VsType {
    VsType::Object
  }
  fn val_to_string(&self) -> String {
    "function Number() { [native code] }".to_string()
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
  fn to_primitive(&self) -> Val {
    Val::String(Rc::new("function Number() { [native code] }".to_string()))
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
    LoadFunctionResult::NativeFunction(to_number)
  }

  fn sub(&self, key: Val) -> Val {
    match key.val_to_string().as_str() {
      "MAX_VALUE" => Val::Number(core::f64::MAX),
      "MAX_SAFE_INTEGER" => Val::Number(2f64.powi(53) - 1f64),
      "MIN_SAFE_INTEGER" => Val::Number(-(2f64.powi(53) - 1f64)),
      "MIN_VALUE" => Val::Number(core::f64::MIN_POSITIVE),
      "NEGATIVE_INFINITY" => Val::Number(core::f64::NEG_INFINITY),
      "POSITIVE_INFINITY" => Val::Number(core::f64::INFINITY),
      "NaN" => Val::Number(core::f64::NAN),
      "isFinite" => Val::Static(&IS_FINITE),
      "isInteger" => Val::Static(&IS_INTEGER),
      "isNaN" => Val::Static(&IS_NAN),
      "isSafeInteger" => Val::Static(&IS_SAFE_INTEGER),
      "parseFloat" => Val::Static(&PARSE_FLOAT),
      "parseInt" => Val::Static(&PARSE_INT),
      _ => Val::Undefined,
    }
  }

  fn submov(&mut self, _key: Val, _value: Val) {
    std::panic!("Not implemented: exceptions");
  }

  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "\x1b[36m[Number]\x1b[39m")
  }

  fn codify(&self) -> String {
    "Number".into()
  }
}

static IS_FINITE: NativeFunction = NativeFunction {
  fn_: |_this: &mut Val, params: Vec<Val>| -> Val {
    if let Some(value) = params.get(0) {
      let number = value.to_number();
      Val::Bool(number.is_finite())
    } else {
      Val::Bool(false)
    }
  },
};

static IS_INTEGER: NativeFunction = NativeFunction {
  fn_: |_this: &mut Val, params: Vec<Val>| -> Val {
    let num = match params.get(0) {
      Some(n) => n.to_number(),
      None => return Val::Bool(false),
    };

    let is_finite = num.is_finite();
    let is_integer = num.floor() == num;

    Val::Bool(is_finite && is_integer)
  },
};

static IS_NAN: NativeFunction = NativeFunction {
  fn_: |_this: &mut Val, params: Vec<Val>| -> Val {
    if let Some(value) = params.get(0) {
      let number = value.to_number();
      Val::Bool(number.is_nan())
    } else {
      Val::Bool(false)
    }
  },
};

static IS_SAFE_INTEGER: NativeFunction = NativeFunction {
  fn_: |_this: &mut Val, params: Vec<Val>| -> Val {
    let num = match params.get(0) {
      Some(n) => n.to_number(),
      None => return Val::Bool(false),
    };

    let is_finite = num.is_finite();
    let is_integer = num.floor() == num;
    let min_safe_integer = -(2f64.powi(53) - 1f64);
    let max_safe_integer = 2f64.powi(53) - 1f64;
    let in_safe_range = min_safe_integer <= num && num <= max_safe_integer;

    Val::Bool(is_finite && is_integer && in_safe_range)
  },
};

static PARSE_FLOAT: NativeFunction = NativeFunction {
  fn_: |_this: &mut Val, params: Vec<Val>| -> Val {
    if let Some(value) = params.get(0) {
      let string_value = value.val_to_string().trim().to_string();

      match string_value.parse::<f64>() {
        Ok(number) => Val::Number(number),
        Err(_) => Val::Number(core::f64::NAN),
      }
    } else {
      Val::Number(core::f64::NAN)
    }
  },
};

static PARSE_INT: NativeFunction = NativeFunction {
  fn_: |_this: &mut Val, params: Vec<Val>| -> Val {
    if let Some(value) = params.get(0) {
      let string_value = value.val_to_string().trim_start().to_string();
      let radix = params.get(1).and_then(|v| v.to_index()).unwrap_or(10);

      if radix < 2 || radix > 36 {
        return Val::Number(core::f64::NAN);
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
    }
  },
};

fn to_number(_: &mut Val, params: Vec<Val>) -> Val {
  if let Some(value) = params.get(0) {
    Val::Number(value.to_number())
  } else {
    Val::Number(0.0)
  }
}