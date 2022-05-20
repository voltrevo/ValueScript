use std::collections::BTreeMap;

use super::vs_value::{Val, ValTrait};
use super::operations::op_sub;

#[derive(Clone, Default)]
pub struct VsObject {
  pub string_map: BTreeMap<String, Val>,
  pub prototype: Option<Val>,
}

impl VsObject {
  pub fn sub(&self, key: Val) -> Val {
    return match self.string_map.get(&key.val_to_string()) {
      Some(val) => val.clone(),
      None => match &self.prototype {
        Some(prototype) => op_sub(prototype.clone(), key),
        None => Val::Undefined,
      },
    };
  }
}
