use std::fmt;
use std::rc::Rc;

use num_bigint::BigInt;

use crate::builtins::error_builtin::ToError;
use crate::builtins::type_error_builtin::ToTypeError;
use crate::stack_frame::StackFrame;
use crate::vs_array::VsArray;
use crate::vs_class::VsClass;
use crate::vs_value::{LoadFunctionResult, Val, ValTrait, VsType};

pub struct NativeFrameFunction {
  pub make_frame: fn() -> StackFrame,
}

impl ValTrait for NativeFrameFunction {
  fn typeof_(&self) -> VsType {
    VsType::Function
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
    panic!("Not implemented");
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
    LoadFunctionResult::StackFrame((self.make_frame)())
  }

  fn sub(&self, _key: Val) -> Result<Val, Val> {
    Err("TODO: Subscript native function".to_error())
  }

  fn submov(&mut self, _key: Val, _value: Val) -> Result<(), Val> {
    Err("Cannot assign to subscript of native function".to_type_error())
  }

  fn pretty_fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "\x1b[36m[Function]\x1b[39m")
  }

  fn codify(&self) -> String {
    "function() { [native code] }".into()
  }
}

impl fmt::Display for NativeFrameFunction {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "function() {{ [native code] }}")
  }
}
