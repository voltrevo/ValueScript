use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::mem::take;

use crate::asm::Module;
use crate::asm::{Definition, DefinitionContent, Pointer};
use crate::visit_pointers::{visit_pointers, PointerVisitation};

pub fn shake_tree(module: &mut Module) {
  let mut dependency_graph = BTreeMap::<Pointer, BTreeSet<Pointer>>::new();
  let mut reverse_dependency_graph = BTreeMap::<Pointer, BTreeSet<Pointer>>::new();
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
  dependency_graph: &BTreeMap<Pointer, BTreeSet<Pointer>>,
  reverse_dependency_graph: &BTreeMap<Pointer, BTreeSet<Pointer>>,
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
  dependency_graph: &BTreeMap<Pointer, BTreeSet<Pointer>>,
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
