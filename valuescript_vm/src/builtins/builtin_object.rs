use std::{fmt, rc::Rc};

use num_bigint::BigInt;

use crate::{
  vs_array::VsArray,
  vs_class::VsClass,
  vs_value::{Val, VsType},
  LoadFunctionResult, ValTrait,
};

use super::type_error_builtin::ToTypeError;

pub trait BuiltinObject: fmt::Display {
  fn bo_name() -> &'static str;
  fn bo_sub(key: &str) -> Val;
  fn bo_load_function() -> LoadFunctionResult;
  fn bo_as_class_data() -> Option<Rc<VsClass>>;
}

impl<T> ValTrait for T
where
  T: BuiltinObject,
{
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

  fn as_class_data(&self) -> Option<Rc<VsClass>> {
    Self::bo_as_class_data()
  }

  fn load_function(&self) -> LoadFunctionResult {
    Self::bo_load_function()
  }

  fn sub(&self, key: Val) -> Result<Val, Val> {
    Ok(Self::bo_sub(&key.to_string()))
  }

  fn submov(&mut self, _key: Val, _value: Val) -> Result<(), Val> {
    Err(format!("Cannot assign to subscript of {} builtin", Self::bo_name()).to_type_error())
  }

  fn pretty_fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "\x1b[36m[{}]\x1b[39m", Self::bo_name())
  }

  fn codify(&self) -> String {
    Self::bo_name().into()
  }
}
