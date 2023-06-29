use std::collections::HashMap;

use crate::{
  asm::{DefinitionContent, Module, Pointer, Value},
  visit_pointers::{visit_pointers, PointerVisitation},
};

pub fn collapse_pointers_of_pointers(module: &mut Module) {
  let mut double_pointer_map = HashMap::<Pointer, Pointer>::new();

  for definition in &mut module.definitions {
    let pointer = match &definition.content {
      DefinitionContent::Value(Value::Pointer(pointer)) => pointer,
      _ => continue,
    };

    double_pointer_map.insert(definition.pointer.clone(), pointer.clone());
  }

  visit_pointers(module, |visitation| match visitation {
    PointerVisitation::Definition(_) => {}
    PointerVisitation::Export(pointer) | PointerVisitation::Reference(_, pointer) => {
      let mut mapped_pointer: &Pointer = pointer;

      loop {
        if let Some(new_pointer) = double_pointer_map.get(mapped_pointer) {
          mapped_pointer = new_pointer;
          continue;
        }

        break;
      }

      *pointer = mapped_pointer.clone();
    }
  });
}
