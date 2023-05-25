use std::rc::Rc;

use num_bigint::BigInt;

use crate::{
  builtins::type_error_builtin::to_type_error,
  native_function::ThisWrapper,
  type_error,
  vs_array::VsArray,
  vs_class::VsClass,
  vs_object::VsObject,
  vs_value::{LoadFunctionResult, Val, VsType},
  ValTrait,
};

pub struct BooleanBuiltin {}

pub static BOOLEAN_BUILTIN: BooleanBuiltin = BooleanBuiltin {};

impl ValTrait for BooleanBuiltin {
  fn typeof_(&self) -> VsType {
    VsType::Object
  }
  fn val_to_string(&self) -> String {
    "function Boolean() { [native code] }".to_string()
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
    Val::String(Rc::new("function Boolean() { [native code] }".to_string()))
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
    LoadFunctionResult::NativeFunction(to_boolean)
  }

  fn sub(&self, _key: Val) -> Result<Val, Val> {
    Ok(Val::Undefined)
  }

  fn submov(&mut self, _key: Val, _value: Val) -> Result<(), Val> {
    type_error!("Cannot assign to subscript of Boolean builtin")
  }

  fn next(&mut self) -> LoadFunctionResult {
    LoadFunctionResult::NotAFunction
  }

  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "\x1b[36m[Boolean]\x1b[39m")
  }

  fn codify(&self) -> String {
    "Boolean".into()
  }
}

fn to_boolean(_: ThisWrapper, params: Vec<Val>) -> Result<Val, Val> {
  Ok(if let Some(value) = params.get(0) {
    Val::Bool(value.is_truthy())
  } else {
    Val::Bool(false)
  })
}
