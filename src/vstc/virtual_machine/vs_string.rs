use std::rc::Rc;

use super::vs_value::Val;
use super::vs_value::VsType;
use super::vs_value::VsValue;

pub struct VsString {
  value: String,
}

impl VsString {
  pub fn from_str(value: &str) -> Val {
    return Rc::new(VsString { value: value.to_string() });
  }

  pub fn from_string(value: String) -> Val {
    return Rc::new(VsString { value: value });
  }
}

impl VsValue for VsString {
  fn typeof_(&self) -> VsType {
    return VsType::String;
  }

  fn to_string(&self) -> String {
    return self.value.clone();
  }

  fn to_number(&self) -> f64 {
    std::panic!("not implemented");
  }

  fn is_primitive(&self) -> bool {
    return true;
  }
}