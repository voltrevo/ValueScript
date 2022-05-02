use std::rc::Rc;

use super::vs_value::Val;
use super::vs_value::VsType;
use super::vs_value::VsValue;
use super::virtual_machine::VirtualMachine;

pub struct VsBool {
  value: bool,
}

impl VsBool {
  pub fn from_bool(value: bool) -> Val {
    return Rc::new(VsBool { value: value });
  }
}

impl VsValue for VsBool {
  fn typeof_(&self) -> VsType {
    return VsType::Bool;
  }

  fn to_string(&self) -> String {
    return self.value.to_string();
  }

  fn to_number(&self) -> f64 {
    return if self.value { 1_f64 } else { 0_f64 };
  }

  fn is_primitive(&self) -> bool {
    return true;
  }

  fn push_frame(&self, vm: &mut VirtualMachine) -> bool {
    return false;
  }

  fn is_truthy(&self) -> bool {
    return self.value;
  }
}
