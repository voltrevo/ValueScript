use std::collections::BTreeMap;

use super::operations::op_sub;
use super::vs_value::{Val, ValTrait};

#[derive(Clone, Default, Debug)]
pub struct VsObject {
  pub string_map: BTreeMap<String, Val>,
  pub prototype: Option<Val>,
}

impl VsObject {
  pub fn sub(&self, key: Val) -> Val {
    return match self.string_map.get(&key.val_to_string()) {
      Some(val) => val.clone(),
      None => match &self.prototype {
        Some(prototype) => op_sub(prototype.clone(), key)
          .map_err(|e| e.val_to_string())
          .unwrap(), // TODO: Exception
        None => Val::Undefined,
      },
    };
  }
}
