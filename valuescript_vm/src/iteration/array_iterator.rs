use std::{fmt, rc::Rc};

use num_bigint::BigInt;

use crate::{
  builtins::type_error_builtin::ToTypeError,
  vs_array::VsArray,
  vs_class::VsClass,
  vs_value::{Val, VsType},
  LoadFunctionResult, ValTrait,
};

struct ArrayIterator {
  array: Rc<VsArray>,
  index: usize,
}

impl ValTrait for ArrayIterator {
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

  fn as_class_data(&self) -> Option<Rc<VsClass>> {
    None
  }

  fn load_function(&self) -> LoadFunctionResult {
    LoadFunctionResult::NotAFunction
  }

  fn sub(&self, _key: Val) -> Result<Val, Val> {
    todo!()
  }

  fn submov(&mut self, _key: Val, _value: Val) -> Result<(), Val> {
    Err("Cannot assign to subscript of array iterator".to_type_error())
  }

  fn pretty_fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "\x1b[36m[ArrayIterator]\x1b[39m")
  }

  fn codify(&self) -> String {
    format!(
      "ArrayIterator({{ array: {}, index: {} }})",
      Val::Array(self.array.clone()).codify(),
      self.index
    )
  }
}

impl fmt::Display for ArrayIterator {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "[object Array Iterator]")
  }
}
