use std::rc::Rc;

use num_bigint::BigInt;

use crate::format_err;
use crate::stack_frame::StackFrame;
use crate::vs_array::VsArray;
use crate::vs_class::VsClass;
use crate::vs_object::VsObject;
use crate::vs_value::{LoadFunctionResult, Val, ValTrait, VsType};
use crate::{builtins::type_error_builtin::to_type_error, type_error};

pub struct NativeFrameFunction {
  pub make_frame: fn() -> StackFrame,
}

impl ValTrait for NativeFrameFunction {
  fn typeof_(&self) -> VsType {
    VsType::Function
  }
  fn val_to_string(&self) -> String {
    "function() { [native code] }".to_string()
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
  fn to_primitive(&self) -> Val {
    Val::String(Rc::new(self.val_to_string()))
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
  fn as_object_data(&self) -> Option<Rc<VsObject>> {
    None
  }
  fn as_class_data(&self) -> Option<Rc<VsClass>> {
    None
  }

  fn load_function(&self) -> LoadFunctionResult {
    LoadFunctionResult::StackFrame((self.make_frame)())
  }

  fn sub(&self, _key: Val) -> Result<Val, Val> {
    format_err!("TODO: Subscript native function")
  }

  fn submov(&mut self, _key: Val, _value: Val) -> Result<(), Val> {
    type_error!("Cannot assign to subscript of native function")
  }

  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "\x1b[36m[Function]\x1b[39m")
  }

  fn codify(&self) -> String {
    "function() { [native code] }".into()
  }
}
