use std::{collections::HashMap, mem::take};

use crate::{
  asm::{
    Definition, DefinitionContent, FnLine, InstructionFieldMut, Module, Number, Pointer, Value,
  },
  name_allocator::NameAllocator,
};

pub fn extract_constants(module: &mut Module, pointer_allocator: &mut NameAllocator) {
  let mut constants = HashMap::<Value, Pointer>::new();

  for defn in &mut module.definitions {
    if let DefinitionContent::Function(f) = &mut defn.content {
      for line in &mut f.body {
        if let FnLine::Instruction(instr) = line {
          instr.visit_fields_mut(&mut |field| match field {
            InstructionFieldMut::Value(value) => {
              value.visit_values_mut(&mut |sub_value| {
                if let Some(p) = constants.get(sub_value) {
                  *sub_value = Value::Pointer(p.clone());
                  return;
                }

                if let Some(name) = should_extract_value_as_constant(sub_value) {
                  let p = Pointer {
                    name: pointer_allocator.allocate(&name),
                  };

                  let existing_p = constants.insert(take(sub_value), p.clone());
                  assert!(existing_p.is_none());
                  *sub_value = Value::Pointer(p);
                }
              });
            }
            InstructionFieldMut::Register(_) | InstructionFieldMut::LabelRef(_) => {}
          });
        }
      }
    }
  }

  for (value, pointer) in constants {
    module.definitions.push(Definition {
      pointer,
      content: DefinitionContent::Value(value),
    });
  }
}

fn should_extract_value_as_constant(value: &Value) -> Option<String> {
  if !is_constant(value) {
    return None;
  }

  match value {
    Value::Void
    | Value::Undefined
    | Value::Null
    | Value::Bool(..)
    | Value::Number(Number(..))
    | Value::Pointer(..)
    | Value::Builtin(..)
    | Value::Register(..) => None,
    Value::BigInt(bi) => {
      let (_, bytes) = bi.to_bytes_le();

      // 2 extra bytes are needed for sign and byte len, so the idea is to
      // allow inlining if it's encoded in 8 bytes (ie size of f64). No idea
      // what's actually optimal here.
      if bytes.len() > 6 {
        Some("bigint".to_string())
      } else {
        None
      }
    }
    Value::String(s) => {
      if s.len() >= 4 {
        Some(mangle_string(s))
      } else {
        None
      }
    }
    Value::Array(array) => {
      if !array.values.is_empty() && array.values.iter().all(is_constant) {
        Some("array".to_string())
      } else {
        None
      }
    }
    Value::Object(object) => {
      if !object.properties.is_empty()
        && object
          .properties
          .iter()
          .all(|(k, v)| is_constant(k) && is_constant(v))
      {
        Some("object".to_string())
      } else {
        None
      }
    }
  }
}

fn is_constant(value: &Value) -> bool {
  match value {
    Value::Void
    | Value::Undefined
    | Value::Null
    | Value::Bool(..)
    | Value::Number(Number(..))
    | Value::BigInt(..)
    | Value::String(..)
    | Value::Pointer(..)
    | Value::Builtin(..) => true,
    Value::Register(..) => false,
    Value::Array(array) => array.values.iter().all(is_constant),
    Value::Object(object) => object
      .properties
      .iter()
      .all(|(k, v)| is_constant(k) && is_constant(v)),
  }
}

fn mangle_string(s: &str) -> String {
  let mut res = "s_".to_string();

  for c in s.chars() {
    if c.is_ascii_alphanumeric() {
      res.push(c);
    } else {
      res.push('_');
    }
  }

  res
}
