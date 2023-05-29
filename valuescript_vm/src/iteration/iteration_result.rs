use std::{fmt, rc::Rc};

use num_bigint::BigInt;

use crate::{
  builtins::type_error_builtin::ToTypeError,
  vs_array::VsArray,
  vs_class::VsClass,
  vs_value::{ToVal, Val, VsType},
  LoadFunctionResult, ValTrait,
};

#[derive(Clone)]
pub struct IterationResult {
  pub value: Val,
  pub done: bool,
}

impl ValTrait for IterationResult {
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

  fn sub(&self, key: Val) -> Result<Val, Val> {
    Ok(match key.to_string().as_str() {
      "value" => self.value.clone(),
      "done" => self.done.to_val(),
      _ => Val::Undefined,
    })
  }

  fn submov(&mut self, _key: Val, _value: Val) -> Result<(), Val> {
    Err("Cannot assign to subscript of iteration result".to_type_error())
  }

  fn pretty_fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self.done {
      false => write!(f, "Iteration({})", self.value.pretty()),
      true => write!(f, "IterationDone({})", self.value.pretty()),
    }
  }

  fn codify(&self) -> String {
    format!("{{ value: {}, done: {} }}", self.value.codify(), self.done)
  }
}

impl fmt::Display for IterationResult {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.write_str("[object Object]")
  }
}
