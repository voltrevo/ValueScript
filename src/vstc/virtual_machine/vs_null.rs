use std::rc::Rc;

use super::vs_value::Val;
use super::vs_value::VsType;
use super::vs_value::VsValue;
use super::virtual_machine::StackFrame;

pub struct VsNull {}

impl VsNull {
  pub fn new() -> Val {
    return Rc::new(VsNull {});
  }
}

impl VsValue for VsNull {
  fn typeof_(&self) -> VsType {
    return VsType::Null;
  }

  fn to_string(&self) -> String {
    return "null".to_string();
  }

  fn to_number(&self) -> f64 {
    return 0_f64;
  }

  fn is_primitive(&self) -> bool {
    return true;
  }

  fn make_frame(&self) -> Option<StackFrame> {
    return None;
  }

  fn is_truthy(&self) -> bool {
    return false;
  }
}
