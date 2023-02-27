use std::rc::Rc;

use super::vs_value::{
  Val,
  VsType,
  ValTrait,
  LoadFunctionResult,
};
use super::vs_object::VsObject;
use super::vs_array::VsArray;
use super::vs_class::VsClass;

pub struct NativeFunction {
  pub fn_: fn(this: &mut Val, params: Vec<Val>) -> Val,
}

impl ValTrait for NativeFunction {
  fn typeof_(&self) -> VsType { VsType::Function }
  fn val_to_string(&self) -> String { "function() { [native code] }".to_string() }
  fn to_number(&self) -> f64 { f64::NAN }
  fn to_index(&self) -> Option<usize> { None }
  fn is_primitive(&self) -> bool { false }
  fn to_primitive(&self) -> Val { Val::String(Rc::new(self.val_to_string())) }
  fn is_truthy(&self) -> bool { true }
  fn is_nullish(&self) -> bool { false }

  fn bind(&self, _params: Vec<Val>) -> Option<Val> {
    std::panic!("Not implemented");
  }

  fn as_array_data(&self) -> Option<Rc<VsArray>> { None }
  fn as_object_data(&self) -> Option<Rc<VsObject>> { None }
  fn as_class_data(&self) -> Option<Rc<VsClass>> { None }

  fn load_function(&self) -> LoadFunctionResult {
    LoadFunctionResult::NativeFunction(self.fn_)
  }

  fn sub(&self, _key: Val) -> Val {
    std::panic!("Not implemented");
  }

  fn submov(&mut self, _key: Val, _value: Val) {
    std::panic!("Not implemented: exceptions");
  }

  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "\x1b[36m[Function]\x1b[39m")
  }

  fn codify(&self) -> String {
    "function() { [native code] }".into()
  }
}
