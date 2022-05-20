use super::vs_value::Val;
use super::vs_object::VsObject;

#[derive(Clone)]
pub struct VsArray {
  pub elements: Vec<Val>,
  pub object: VsObject,
}

impl VsArray {
  pub fn from(vals: Vec<Val>) -> VsArray {
    return VsArray {
      elements: vals,
      object: VsObject::default(),
    };
  }
}
