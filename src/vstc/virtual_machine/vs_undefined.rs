use std::rc::Rc;

use super::vs_value::Val;
use super::vs_value::VsType;
use super::vs_value::VsValue;
use super::virtual_machine::StackFrame;

pub struct VsUndefined {}

impl VsUndefined {
  pub fn new() -> Val {
    return Rc::new(VsUndefined {});
  }
}

impl VsValue for VsUndefined {
  fn typeof_(&self) -> VsType {
    return VsType::Undefined;
  }

  fn to_string(&self) -> String {
    return "undefined".to_string();
  }

  fn to_number(&self) -> f64 {
    return f64::NAN;
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
