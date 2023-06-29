use std::collections::{HashMap, HashSet};
use std::mem::take;

use crate::asm::Module;
use crate::asm::{Definition, DefinitionContent, FnLine, InstructionFieldMut, Pointer, Value};
use crate::name_allocator::NameAllocator;
use crate::visit_pointers::{visit_pointers, PointerVisitation};

pub fn optimize(module: &mut Module, pointer_allocator: &mut NameAllocator) {
  collapse_pointers_of_pointers(module);
  shake_tree(module);
  extract_constants(module, pointer_allocator);
}

fn collapse_pointers_of_pointers(module: &mut Module) {
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

fn shake_tree(module: &mut Module) {
  let mut dependency_graph = HashMap::<Pointer, HashSet<Pointer>>::new();
  let mut pointers_to_include = Vec::<Pointer>::new();

  visit_pointers(module, |visitation| match visitation {
    PointerVisitation::Export(exported_pointer) => {
      pointers_to_include.push(exported_pointer.clone());
    }
    PointerVisitation::Definition(_) => {}
    PointerVisitation::Reference(owner, pointer) => {
      dependency_graph
        .entry(owner.clone())
        .or_default()
        .insert(pointer.clone());
    }
  });

  let mut pointers_included = HashSet::<Pointer>::new();
  let mut pointers_to_include_i = 0;

  while pointers_to_include_i < pointers_to_include.len() {
    let pointer = &pointers_to_include[pointers_to_include_i];
    pointers_to_include_i += 1;

    pointers_included.insert(pointer.clone());

    if let Some(dependencies) = dependency_graph.get(pointer) {
      // TODO: Avoid randomness caused by HashSet iteration
      for dependency in dependencies {
        if !pointers_included.contains(dependency) {
          pointers_to_include.push(dependency.clone());
        }
      }
    }
  }

  let previous_definitions = std::mem::take(&mut module.definitions);
  let mut new_definitions_map = HashMap::<Pointer, Definition>::new();

  for definition in previous_definitions {
    if pointers_included.contains(&definition.pointer) {
      new_definitions_map.insert(definition.pointer.clone(), definition);
    }
  }

  let mut new_definitions = Vec::<Definition>::new();

  for pointer in &pointers_to_include {
    let defn = new_definitions_map.get_mut(pointer).unwrap();

    if let DefinitionContent::Value(_) = defn.content {
      // Exclude values on the first pass. TODO: Proper depth-first search (+reverse) to ensure
      // correct ordering.
      continue;
    }

    if defn.pointer.name != "" {
      new_definitions.push(take(defn));
    }
  }

  for pointer in pointers_to_include {
    let defn = new_definitions_map.get_mut(&pointer).unwrap();

    if defn.pointer.name != "" {
      new_definitions.push(take(defn));
    }
  }

  module.definitions = new_definitions;
}

fn extract_constants(module: &mut Module, pointer_allocator: &mut NameAllocator) {
  let mut constants = HashMap::<Value, Pointer>::new();

  for defn in &mut module.definitions {
    if let DefinitionContent::Function(f) = &mut defn.content {
      for line in &mut f.body {
        if let FnLine::Instruction(instr) = line {
          instr.visit_fields_mut(&mut |field| match field {
            InstructionFieldMut::Value(value) => {
              value.visit_values_mut(&mut |sub_value| {
                if let Some(p) = constants.get(&sub_value) {
                  *sub_value = Value::Pointer(p.clone());
                  return;
                }

                if let Some(name) = should_extract_value_as_constant(&sub_value) {
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
  if let Value::String(s) = value {
    if s.len() >= 4 {
      return Some(mangle_string(s));
    }
  }

  None
}

fn mangle_string(s: &String) -> String {
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
