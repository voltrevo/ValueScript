use std::rc::Rc;

use super::vs_value::Val;
use super::vs_value::VsType;
use super::vs_value::VsValue;
use super::virtual_machine::VirtualMachine;

pub struct VsNumber {
  value: f64,
}

impl VsNumber {
  pub fn from_f64(value: f64) -> Val {
    return Rc::new(VsNumber { value: value });
  }
}

impl VsValue for VsNumber {
  fn typeof_(&self) -> VsType {
    return VsType::Number;
  }

  fn to_string(&self) -> String {
    return self.value.to_string();
  }

  fn to_number(&self) -> f64 {
    return self.value;
  }

  fn is_primitive(&self) -> bool {
    return true;
  }

  fn push_frame(&self, vm: &mut VirtualMachine) -> bool {
    return false;
  }
}