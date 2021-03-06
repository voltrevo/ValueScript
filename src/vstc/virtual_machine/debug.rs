use std::rc::Rc;

use super::vs_value::{Val, VsType, ValTrait, LoadFunctionResult};
use super::vs_array::VsArray;
use super::vs_object::VsObject;
use super::vs_class::VsClass;
use super::native_function::NativeFunction;

pub struct Debug {}

pub static DEBUG: Debug = Debug {};

impl ValTrait for Debug {
  fn typeof_(&self) -> VsType { VsType::Object }
  fn val_to_string(&self) -> String { "[object Debug]".to_string() }
  fn to_number(&self) -> f64 { f64::NAN }
  fn to_index(&self) -> Option<usize> { None }
  fn is_primitive(&self) -> bool { false }
  fn to_primitive(&self) -> Val { Val::String(Rc::new(self.val_to_string())) }
  fn is_truthy(&self) -> bool { true }
  fn is_nullish(&self) -> bool { false }

  fn bind(&self, _params: Vec<Val>) -> Option<Val> { None }

  fn as_array_data(&self) -> Option<Rc<VsArray>> { None }
  fn as_object_data(&self) -> Option<Rc<VsObject>> { None }
  fn as_class_data(&self) -> Option<Rc<VsClass>> { None }

  fn load_function(&self) -> LoadFunctionResult {
    LoadFunctionResult::NotAFunction
  }

  fn sub(&self, key: Val) -> Val {
    match key.val_to_string().as_str() {
      "log" => Val::Static(&LOG),

      _ => Val::Undefined,
    }
  }

  fn submov(&mut self, _key: Val, _value: Val) {
    std::panic!("Not implemented: exceptions");
  }

  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "\x1b[36m[Debug]\x1b[39m")
  }
}

static LOG: NativeFunction = NativeFunction {
  fn_: |_this: &mut Val, params: Vec<Val>| -> Val {
    for p in params {
      println!("Debug.log: {}", p);
    }

    return Val::Undefined;
  }
};
