use std::collections::{HashMap, HashSet};
use std::mem::take;

use crate::asm::{Definition, DefinitionContent, FnLine, InstructionFieldMut, Pointer, Value};
use crate::asm::{Module, Number};
use crate::name_allocator::NameAllocator;
use crate::visit_pointers::{visit_pointers, PointerVisitation};

pub fn optimize(module: &mut Module, pointer_allocator: &mut NameAllocator) {
  collapse_pointers_of_pointers(module);
  extract_constants(module, pointer_allocator);
  shake_tree(module);
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
  let mut reverse_dependency_graph = HashMap::<Pointer, HashSet<Pointer>>::new();
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

      reverse_dependency_graph
        .entry(pointer.clone())
        .or_default()
        .insert(owner.clone());
    }
  });

  let mut required_pointers = HashSet::<Pointer>::new();

  for p in &pointers_to_include {
    gather_required_pointers(p, &mut required_pointers, &dependency_graph)
  }

  let mut ordered_pointers = Vec::<Pointer>::new();
  let mut pointers_in_progress = HashSet::<Pointer>::new();

  for p in pointers_to_include {
    include_pointer(
      &p,
      &mut ordered_pointers,
      &mut dependency_graph,
      &mut reverse_dependency_graph,
      &mut pointers_in_progress,
      &required_pointers,
    );
  }

  let previous_definitions = std::mem::take(&mut module.definitions);
  let mut new_definitions_map = HashMap::<Pointer, Definition>::new();

  for definition in previous_definitions {
    if required_pointers.contains(&definition.pointer) {
      new_definitions_map.insert(definition.pointer.clone(), definition);
    }
  }

  let mut new_definitions = Vec::<Definition>::new();

  for pointer in &ordered_pointers {
    if required_pointers.contains(pointer) {
      let defn = new_definitions_map.get_mut(pointer).unwrap();

      // First include pointers that are allowed to be circular
      match &defn.content {
        DefinitionContent::Function(..) | DefinitionContent::Class(..) => {}
        DefinitionContent::Value(..) | DefinitionContent::Lazy(..) => continue,
      }

      new_definitions.push(take(defn));
    }
  }

  for pointer in ordered_pointers {
    if required_pointers.contains(&pointer) {
      let defn = new_definitions_map.get_mut(&pointer).unwrap();

      if defn.pointer.name == "" {
        // "" isn't a valid pointer name - this happens when `take` has already been used on the
        // definition
        continue;
      }

      new_definitions.push(take(defn));
    }
  }

  module.definitions = new_definitions;
}

fn include_pointer(
  p: &Pointer,
  ordered_pointers: &mut Vec<Pointer>,
  dependency_graph: &HashMap<Pointer, HashSet<Pointer>>,
  reverse_dependency_graph: &HashMap<Pointer, HashSet<Pointer>>,
  pointers_in_progress: &mut HashSet<Pointer>,
  required_pointers: &HashSet<Pointer>,
) {
  if pointers_in_progress.contains(p) {
    return;
  }

  pointers_in_progress.insert(p.clone());

  // include things that depend on p
  if let Some(rev_deps) = reverse_dependency_graph.get(p) {
    for rev_dep in rev_deps {
      if required_pointers.contains(rev_dep) {
        include_pointer(
          rev_dep,
          ordered_pointers,
          dependency_graph,
          reverse_dependency_graph,
          pointers_in_progress,
          required_pointers,
        );
      }
    }
  }

  // include p
  ordered_pointers.push(p.clone());

  // include p's dependencies
  if let Some(deps) = dependency_graph.get(p) {
    for dep in deps {
      include_pointer(
        dep,
        ordered_pointers,
        dependency_graph,
        reverse_dependency_graph,
        pointers_in_progress,
        required_pointers,
      )
    }
  }
}

fn gather_required_pointers(
  p: &Pointer,
  required_pointers: &mut HashSet<Pointer>,
  dependency_graph: &HashMap<Pointer, HashSet<Pointer>>,
) {
  let inserted = required_pointers.insert(p.clone());

  if !inserted {
    // This pointer is already in progress, so don't redundantly include it.
    // This indicates a circular dependency, which is perfectly fine for functions.
    return;
  }

  if let Some(deps) = dependency_graph.get(p) {
    for dep in deps {
      gather_required_pointers(dep, required_pointers, dependency_graph);
    }
  }
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
  if !is_constant(value) {
    return None;
  }

  match value {
    Value::Void
    | Value::Undefined
    | Value::Null
    | Value::Bool(..)
    | Value::Number(Number(..))
    | Value::BigInt(..)
    | Value::Pointer(..)
    | Value::Builtin(..)
    | Value::Register(..) => None,
    Value::String(s) => {
      if s.len() >= 4 {
        Some(mangle_string(s))
      } else {
        None
      }
    }
    Value::Array(array) => {
      if array.values.iter().all(is_constant) {
        Some("array".to_string())
      } else {
        None
      }
    }
    Value::Object(object) => {
      if object
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
