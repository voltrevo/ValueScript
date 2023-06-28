use std::rc::Rc;

use crate::vs_value::{ToVal, Val};

#[derive(Clone, Debug)]
pub struct VsArray {
  pub elements: Vec<Val>,
}

impl VsArray {
  pub fn from(vals: Vec<Val>) -> VsArray {
    return VsArray { elements: vals };
  }

  pub fn new() -> VsArray {
    return VsArray { elements: vec![] };
  }
}

impl ToVal for VsArray {
  fn to_val(self) -> Val {
    Val::Array(Rc::new(self))
  }
}
