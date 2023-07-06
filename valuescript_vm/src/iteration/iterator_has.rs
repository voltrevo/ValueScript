use crate::{vs_value::Val, VsSymbol};

pub fn iterator_has(key: &Val) -> Option<bool> {
  if key.to_string() == "next" {
    return Some(true);
  }

  if let Val::Symbol(key) = key {
    match key {
      VsSymbol::ITERATOR => {
        return Some(true);
      }
    }
  }

  Some(false)
}
