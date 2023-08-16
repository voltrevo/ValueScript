use std::collections::BTreeMap;

use valuescript_vm::{
  vs_class::VsClass,
  vs_object::VsObject,
  vs_value::{ToVal, Val},
};

use crate::asm::{ContentHashable, Number, Value};

pub trait TryToVal {
  fn try_to_val(self) -> Result<Val, Val>;
}

impl TryToVal for Value {
  fn try_to_val(self) -> Result<Val, Val> {
    Ok(match self {
      Value::Undefined => Val::Undefined,
      Value::Null => Val::Null,
      Value::Bool(b) => b.to_val(),
      Value::Number(Number(n)) => n.to_val(),
      Value::BigInt(n) => n.to_val(),
      Value::String(s) => s.to_val(),
      Value::Array(arr) => {
        let mut result = Vec::<Val>::new();

        for value in arr.values {
          result.push(value.try_to_val()?);
        }

        result.to_val()
      }
      Value::Object(obj) => {
        let mut string_map = BTreeMap::<String, Val>::new();

        for (key, value) in obj.properties {
          string_map.insert(key.try_to_val()?.to_string(), value.try_to_val()?);
        }

        VsObject {
          string_map,
          symbol_map: Default::default(),
          prototype: None,
        }
        .to_val()
      }
      Value::Class(class) => VsClass {
        name: class.meta.name,
        content_hash: match class.meta.content_hashable {
          ContentHashable::Empty | ContentHashable::Src(_, _) => None,
          ContentHashable::Content(hash) => Some(hash.0),
        },
        constructor: class.constructor.try_to_val()?,
        prototype: class.prototype.try_to_val()?,
        static_: class.static_.try_to_val()?,
      }
      .to_val(),

      Value::Void | Value::Register(..) | Value::Pointer(..) | Value::Builtin(..) => {
        return Err("Invalid argument".to_val());
      }
    })
  }
}
