use valuescript_vm::{vs_value::Val, VsSymbol};

use crate::asm::{Builtin, Number};

use super::kal::{Array, Kal, Object};

pub trait TryToKal {
  fn try_to_kal(&self) -> Option<Kal>;
}

impl TryToKal for Val {
  fn try_to_kal(&self) -> Option<Kal> {
    Some(match self {
      Val::Void => Kal::Undefined,
      Val::Undefined => Kal::Undefined,
      Val::Null => Kal::Null,
      Val::Bool(b) => Kal::Bool(*b),
      Val::Number(n) => Kal::Number(Number(*n)),
      Val::BigInt(n) => Kal::BigInt(n.clone()),
      Val::Symbol(sym) => match sym {
        VsSymbol::ITERATOR => Kal::Builtin(Builtin {
          name: "SymbolIterator".to_string(),
        }),
      },
      Val::String(s) => Kal::String(s.to_string()),
      Val::Array(arr) => {
        let mut values = Vec::<Kal>::new();

        for value in &arr.elements {
          values.push(value.try_to_kal()?);
        }

        Kal::Array(Box::new(Array { values }))
      }
      Val::Object(obj) => {
        if obj.prototype.is_some() {
          // TODO: convert object with prototype to Kal
          return None;
        }

        let mut properties = Vec::<(Kal, Kal)>::new();

        for (k, v) in &obj.symbol_map {
          let k = match k {
            VsSymbol::ITERATOR => Kal::Builtin(Builtin {
              name: "SymbolIterator".to_string(),
            }),
          };

          properties.push((k, v.try_to_kal()?));
        }

        for (k, v) in &obj.string_map {
          properties.push((Kal::String(k.clone()), v.try_to_kal()?));
        }

        Kal::Object(Box::new(Object { properties }))
      }
      // TODO: support more of these
      Val::Function(..)
      | Val::Class(..)
      | Val::Static(..)
      | Val::Dynamic(..)
      | Val::CopyCounter(..) => return None,
      Val::StoragePtr(ptr) => return ptr.get().try_to_kal(),
    })
  }
}
