use super::vs_value::Val;

#[derive(Debug)]
pub struct VsClass {
  pub constructor: Val,
  pub instance_prototype: Val,
}

impl VsClass {}
