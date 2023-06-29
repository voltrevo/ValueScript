use valuescript_vm::{
  vs_value::{ToVal, Val},
  VsSymbol,
};

use crate::asm::{Array, Builtin, Number, Object, Value};

pub trait TryToValue {
  fn try_to_value(&self) -> Result<Value, Val>;
}

impl TryToValue for Val {
  fn try_to_value(&self) -> Result<Value, Val> {
    Ok(match self {
      Val::Void => Value::Undefined,
      Val::Undefined => Value::Undefined,
      Val::Null => Value::Null,
      Val::Bool(b) => Value::Bool(*b),
      Val::Number(n) => Value::Number(Number(*n)),
      Val::BigInt(n) => Value::BigInt(n.clone()),
      Val::Symbol(sym) => match sym {
        VsSymbol::ITERATOR => Value::Builtin(Builtin {
          name: "SymbolIterator".to_string(),
        }),
      },
      Val::String(s) => Value::String(s.to_string()),
      Val::Array(arr) => {
        let mut values = Vec::<Value>::new();

        for value in &arr.elements {
          values.push(value.try_to_value()?);
        }

        Value::Array(Box::new(Array { values }))
      }
      Val::Object(obj) => {
        if obj.prototype.is_some() {
          return Err("can't (yet?) convert object with prototype to Value".to_val());
        }

        let mut properties = Vec::<(Value, Value)>::new();

        for (k, v) in &obj.symbol_map {
          let k = match k {
            VsSymbol::ITERATOR => Value::Builtin(Builtin {
              name: "SymbolIterator".to_string(),
            }),
          };

          properties.push((k, v.try_to_value()?));
        }

        for (k, v) in &obj.string_map {
          properties.push((Value::String(k.clone()), v.try_to_value()?));
        }

        Value::Object(Box::new(Object { properties }))
      }
      Val::Function(..)
      | Val::Class(..)
      | Val::Static(..)
      | Val::Dynamic(..)
      | Val::CopyCounter(..) => return Err("TODO: support more of these".to_val()),
    })
  }
}
