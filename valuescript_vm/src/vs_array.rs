use std::rc::Rc;

use crate::vs_value::{ToVal, Val};

#[derive(Clone, Debug, Default)]
pub struct VsArray {
  pub elements: Vec<Val>, // TODO: VsArray(Vec<Val>)?
}

impl VsArray {
  pub fn from(vals: Vec<Val>) -> VsArray {
    VsArray { elements: vals }
  }
}

impl ToVal for VsArray {
  fn to_val(self) -> Val {
    Val::Array(Rc::new(self))
  }
}
