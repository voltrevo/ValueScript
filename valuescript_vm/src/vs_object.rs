use std::collections::BTreeMap;
use std::rc::Rc;

use crate::vs_symbol::VsSymbol;
use crate::vs_value::ToVal;
use crate::ValTrait;

use super::vs_value::Val;

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
      _ => self.string_map.get(&key.to_string()),
    };

    if let Some(val) = val {
      return val.clone();
    }

    match &self.prototype {
      Some(prototype) => prototype.sub(key).map_err(|e| e.to_string()).unwrap(), // TODO: Exception
      None => Val::Undefined,
    }
  }
}

impl ToVal for VsObject {
  fn to_val(self) -> Val {
    Val::Object(Rc::new(self))
  }
}
