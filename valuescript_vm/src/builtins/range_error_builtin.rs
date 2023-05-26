use std::fmt;
use std::{collections::BTreeMap, rc::Rc};

use num_bigint::BigInt;

use crate::native_function::ThisWrapper;
use crate::vs_value::{ToVal, ToValString};
use crate::{
  native_function::NativeFunction,
  operations::{op_sub, op_submov},
  vs_array::VsArray,
  vs_class::VsClass,
  vs_object::VsObject,
  vs_value::{LoadFunctionResult, Val, VsType},
  ValTrait,
};

use super::type_error_builtin::ToTypeError;

pub struct RangeErrorBuiltin {}

pub static RANGE_ERROR_BUILTIN: RangeErrorBuiltin = RangeErrorBuiltin {};

impl ValTrait for RangeErrorBuiltin {
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
  fn to_primitive(&self) -> Val {
    self.to_val_string()
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
    Some(Rc::new(VsClass {
      constructor: Val::Static(&SET_MESSAGE),
      instance_prototype: make_range_error_prototype(),
    }))
  }

  fn load_function(&self) -> LoadFunctionResult {
    LoadFunctionResult::NativeFunction(to_range_error)
  }

  fn sub(&self, _key: Val) -> Result<Val, Val> {
    Ok(Val::Undefined)
  }

  fn submov(&mut self, _key: Val, _value: Val) -> Result<(), Val> {
    Err("Cannot assign to subscript of RangeError builtin".to_type_error())
  }

  fn next(&mut self) -> LoadFunctionResult {
    LoadFunctionResult::NotAFunction
  }

  fn pretty_fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "\x1b[36m[RangeError]\x1b[39m")
  }

  fn codify(&self) -> String {
    "RangeError".into()
  }
}

impl fmt::Display for RangeErrorBuiltin {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "function RangeError() {{ [native code] }}")
  }
}

pub fn to_range_error(_: ThisWrapper, params: Vec<Val>) -> Result<Val, Val> {
  Ok(
    VsObject {
      string_map: BTreeMap::from([(
        "message".to_string(),
        match params.get(0) {
          Some(param) => param.to_val_string(),
          None => "".to_val(),
        },
      )]),
      symbol_map: Default::default(),
      prototype: Some(make_range_error_prototype()),
    }
    .to_val(),
  )
}

// TODO: Static? (Rc -> Arc?)
fn make_range_error_prototype() -> Val {
  VsObject {
    string_map: BTreeMap::from([
      ("name".to_string(), "RangeError".to_val()),
      ("toString".to_string(), Val::Static(&RANGE_ERROR_TO_STRING)),
    ]),
    symbol_map: Default::default(),
    prototype: None,
  }
  .to_val()
}

static SET_MESSAGE: NativeFunction = NativeFunction {
  fn_: |mut this: ThisWrapper, params: Vec<Val>| -> Result<Val, Val> {
    let message = match params.get(0) {
      Some(param) => param.to_string(),
      None => "".to_string(),
    };

    op_submov(this.get_mut()?, "message".to_val(), message.to_val())?;

    Ok(Val::Undefined)
  },
};

static RANGE_ERROR_TO_STRING: NativeFunction = NativeFunction {
  fn_: |this: ThisWrapper, _params: Vec<Val>| -> Result<Val, Val> {
    let message = op_sub(this.get().clone(), "message".to_val())?;
    Ok(format!("RangeError({})", message).to_val())
  },
};

#[macro_export]
macro_rules! range_error {
  ($fmt:expr $(, $($arg:expr),*)?) => {{
    let formatted_string = format!($fmt $(, $($arg),*)?);
    Err(to_range_error(
      ThisWrapper::new(true, &mut Val::Undefined),
      vec![formatted_string.to_val()],
    ).unwrap())
  }};
}
