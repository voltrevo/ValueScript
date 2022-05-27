use std::rc::Rc;

use super::vs_value::Val;
use super::vs_function::VsFunction;

pub struct VsClass {
  pub constructor: Rc<VsFunction>,
  pub instance_prototype: Val,
}

impl VsClass {}
