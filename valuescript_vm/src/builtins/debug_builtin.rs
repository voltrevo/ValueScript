use std::fmt;
use std::rc::Rc;

use num_bigint::BigInt;

use crate::native_function::{NativeFunction, ThisWrapper};
use crate::vs_array::VsArray;
use crate::vs_class::VsClass;
use crate::vs_object::VsObject;
use crate::vs_value::{LoadFunctionResult, Val, ValTrait, VsType};

use super::type_error_builtin::ToTypeError;

pub struct DebugBuiltin {}

pub static DEBUG_BUILTIN: DebugBuiltin = DebugBuiltin {};

impl ValTrait for DebugBuiltin {
  fn typeof_(&self) -> VsType {
    VsType::Object
  }
  fn to_number(&self) -> f64 {
    f64::NAN
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
    LoadFunctionResult::NotAFunction
  }

  fn sub(&self, key: Val) -> Result<Val, Val> {
    Ok(match key.to_string().as_str() {
      "log" => Val::Static(&LOG),

      _ => Val::Undefined,
    })
  }

  fn submov(&mut self, _key: Val, _value: Val) -> Result<(), Val> {
    Err("Cannot assign to subscript of Debug builtin".to_type_error())
  }

  fn next(&mut self) -> LoadFunctionResult {
    LoadFunctionResult::NotAFunction
  }

  fn pretty_fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "\x1b[36m[Debug]\x1b[39m")
  }

  fn codify(&self) -> String {
    "Debug".into()
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
