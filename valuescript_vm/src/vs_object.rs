use std::collections::BTreeMap;

use crate::vs_symbol::VsSymbol;

use super::operations::op_sub;
use super::vs_value::{Val, ValTrait};

#[derive(Clone, Default, Debug)]
pub struct VsObject {
  pub string_map: BTreeMap<String, Val>,
  pub symbol_map: BTreeMap<VsSymbol, Val>,
  pub prototype: Option<Val>,
}

impl VsObject {
  pub fn sub(&self, key: Val) -> Val {
    let val = match &key {
      Val::String(string) => self.string_map.get(&**string),
      Val::Symbol(symbol) => self.symbol_map.get(symbol),
      _ => self.string_map.get(&key.val_to_string()),
    };

    if let Some(val) = val {
      return val.clone();
    }

    match &self.prototype {
      Some(prototype) => op_sub(prototype.clone(), key)
        .map_err(|e| e.val_to_string())
        .unwrap(), // TODO: Exception
      None => Val::Undefined,
    }
  }
}
