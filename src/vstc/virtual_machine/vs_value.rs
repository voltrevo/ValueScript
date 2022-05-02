use std::rc::Rc;

use super::vs_string::VsString;
use super::virtual_machine::VirtualMachine;

pub type Val = Rc<dyn VsValue>;

#[derive(PartialEq)]
pub enum VsType {
  Undefined,
  Null,
  Bool,
  Number,
  String,
  Array,
  Object,
  Function,
}

impl VsType {
  pub fn as_val(&self) -> Val {
    return VsString::from_str(match self {
      Undefined => "undefined",
      Null => "object",
      Bool => "boolean",
      Number => "number",
      String => "string",
      Array => "object",
      Object => "object",
      Function => "function",
    });
  }
}

pub trait VsValue {
  fn typeof_(&self) -> VsType;
  fn to_string(&self) -> String;
  fn to_number(&self) -> f64;
  fn is_primitive(&self) -> bool;

  fn push_frame(&self, vm: &mut VirtualMachine) -> bool;
}

pub trait ValTrait {
  fn to_primitive(&self) -> Val;
}

impl ValTrait for Val {
  fn to_primitive(&self) -> Val {
    if self.is_primitive() {
      return self.clone();
    }

    return VsString::from_string(self.to_string());
  }
}

impl std::fmt::Display for dyn VsValue {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.to_string())
  }
}
