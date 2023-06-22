use std::rc::Rc;

use crate::vs_value::ToVal;

use super::vs_value::Val;

#[derive(Debug)]
pub struct VsClass {
  pub constructor: Val,
  pub prototype: Val,
}

impl VsClass {}

impl ToVal for VsClass {
  fn to_val(self) -> Val {
    Val::Class(Rc::new(self))
  }
}
