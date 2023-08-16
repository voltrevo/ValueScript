use std::collections::{BTreeMap, HashMap, HashSet};
use std::mem::swap;

use tiny_keccak::{Hasher, Keccak};

use crate::asm::{
  ContentHashable, Definition, DefinitionContent, ExportStar, FnLine, Hash, Instruction, Object,
  Pointer, Structured, Value,
};
use crate::gather_modules::PathAndModule;
use crate::import_pattern::{ImportKind, ImportPattern};
use crate::name_allocator::NameAllocator;
use crate::optimization::optimize;
use crate::resolve_path::{resolve_path, ResolvedPath};
use crate::visit_pointers::{visit_pointers, PointerVisitation};
use crate::DiagnosticLevel;
use crate::{asm::Module, Diagnostic};

pub struct LinkModuleResult {
  pub module: Option<Module>,
  pub diagnostics: Vec<Diagnostic>, // TODO: Associate paths/spans properly
}

pub fn link_module(
  entry_point: &ResolvedPath,
  modules: &HashMap<ResolvedPath, PathAndModule>,
) -> LinkModuleResult {
  let mut result = LinkModuleResult {
    module: None,
    diagnostics: vec![],
  };

  let mut pointer_allocator = NameAllocator::default();
  let mut included_modules = HashMap::<ResolvedPath, (Value, ExportStar)>::new();

  let mut path_and_module = match modules.get(&entry_point.clone()) {
    Some(path_and_module) => path_and_module.clone(),
    None => {
      result.diagnostics.push(Diagnostic {
        level: DiagnosticLevel::Error,
        message: format!("Module not found: {}", entry_point),
        span: swc_common::DUMMY_SP,
      });

      return result;
    }
  };

  let mut modules_to_include = resolve_and_rewrite_import_patterns(&mut path_and_module);
  let mut modules_to_include_i = 0;

  // No rewrites should actually occur here, but we still need to do this to get the names into the
  // allocator.
  rewrite_pointers(&mut path_and_module.module, &mut pointer_allocator);

  included_modules.insert(
    entry_point.clone(),
    (
      path_and_module.module.export_default.clone(),
      path_and_module.module.export_star.clone(),
    ),
  );

  while modules_to_include_i < modules_to_include.len() {
    let module_to_include = modules_to_include[modules_to_include_i].clone();
    modules_to_include_i += 1;

    if included_modules.contains_key(&module_to_include) {
      continue;
    }

    let mut including_path_and_module = match modules.get(&module_to_include) {
      Some(pm) => pm.clone(),
      None => {
        result.diagnostics.push(Diagnostic {
          level: DiagnosticLevel::Error,
          message: format!("Module not found: {}", module_to_include),
          span: swc_common::DUMMY_SP,
        });

        continue;
      }
    };

    let mut new_modules_to_include =
      resolve_and_rewrite_import_patterns(&mut including_path_and_module);

    modules_to_include.append(&mut new_modules_to_include);

    rewrite_pointers(
      &mut including_path_and_module.module,
      &mut pointer_allocator,
    );

    included_modules.insert(
      module_to_include,
      (
        including_path_and_module.module.export_default,
        including_path_and_module.module.export_star,
      ),
    );

    path_and_module
      .module
      .definitions
      .append(&mut including_path_and_module.module.definitions);
  }

  link_import_patterns(
    &mut path_and_module.module,
    &included_modules,
    &mut result.diagnostics,
  );

  collapse_pointers_of_pointers(&mut path_and_module.module);
  calculate_content_hashes(&mut path_and_module.module, &mut result.diagnostics);

  optimize(&mut path_and_module.module, &mut pointer_allocator);

  result.module = Some(path_and_module.module);
  result
}

fn rewrite_pointers(module: &mut Module, pointer_allocator: &mut NameAllocator) {
  let mut pointer_map = HashMap::<Pointer, Pointer>::new();

  for definition in &module.definitions {
    let mapped_pointer = Pointer {
      name: pointer_allocator.allocate(&definition.pointer.name),
    };

    if mapped_pointer != definition.pointer {
      pointer_map.insert(definition.pointer.clone(), mapped_pointer);
    }
  }

  visit_pointers(module, |visitation| match visitation {
    PointerVisitation::Export(pointer)
    | PointerVisitation::Definition(pointer)
    | PointerVisitation::Reference(_, pointer) => {
      if let Some(mapped_pointer) = pointer_map.get(pointer) {
        *pointer = mapped_pointer.clone();
      }
    }
  });
}

fn resolve_and_rewrite_import_patterns(path_and_module: &mut PathAndModule) -> Vec<ResolvedPath> {
  let mut resolved_paths = Vec::<ResolvedPath>::new();

  for definition in &mut path_and_module.module.definitions {
    match ImportPattern::decode(definition) {
      Some(_) => {}
      None => continue,
    }

    let lazy = match &mut definition.content {
      DefinitionContent::Lazy(lazy) => lazy,
      _ => panic!("Inconsistent with import pattern"),
    };

    let first_instruction = match lazy.body.first_mut() {
      Some(FnLine::Instruction(instruction)) => instruction,
      _ => panic!("Inconsistent with import pattern"),
    };

    let import_string = match first_instruction {
      Instruction::Import(Value::String(string), _)
      | Instruction::ImportStar(Value::String(string), _) => string,
      _ => panic!("Inconsistent with import pattern"),
    };

    let resolved = resolve_path(&path_and_module.path, import_string);
    resolved_paths.push(resolved.clone());
    *import_string = resolved.path;
  }

  resolved_paths
}

fn link_import_patterns(
  module: &mut Module,
  included_modules: &HashMap<ResolvedPath, (Value, ExportStar)>,
  diagnostics: &mut Vec<Diagnostic>,
) {
  module.export_star = ExportStar {
    includes: vec![],
    local: flatten_export_star(&module.export_star, &*module, included_modules, diagnostics),
  };

  let mut new_definitions = HashMap::<Pointer, Definition>::new();

  for definition in &module.definitions {
    let import_pattern = match ImportPattern::decode(definition) {
      Some(import_pattern) => import_pattern,
      None => continue,
    };

    let resolved_path = ResolvedPath {
      // Should have been resolved already during resolve_and_rewrite_import_patterns
      path: import_pattern.path.clone(),
    };

    let (default, export_star) = match included_modules.get(&resolved_path) {
      Some(el) => el,
      None => continue,
    };

    let export_star = flatten_export_star(export_star, module, included_modules, diagnostics);

    let new_definition = Definition {
      pointer: import_pattern.pointer.clone(),
      content: match import_pattern.kind {
        ImportKind::Default => DefinitionContent::Value(default.clone()),
        ImportKind::Star => DefinitionContent::Value(Value::Object(Box::new(export_star.clone()))),
        ImportKind::Name(name) => match export_star.try_resolve_key(&name) {
          Some(value) => DefinitionContent::Value(value.clone()),
          None => {
            diagnostics.push(Diagnostic {
              level: DiagnosticLevel::Error,
              message: format!(
                "Imported name `{}` does not exist in `{}`",
                name, import_pattern.path
              ),
              span: swc_common::DUMMY_SP,
            });

            continue;
          }
        },
      },
    };

    new_definitions.insert(import_pattern.pointer, new_definition);
  }

  for definition in &mut module.definitions {
    if let Some(new_definition) = new_definitions.get_mut(&definition.pointer) {
      swap(definition, new_definition);
    }
  }
}

fn flatten_export_star(
  export_star: &ExportStar,
  module: &Module,
  included_modules: &HashMap<ResolvedPath, (Value, ExportStar)>,
  diagnostics: &mut Vec<Diagnostic>,
) -> Object {
  let mut include_pointers_to_process = export_star.includes.clone();
  let mut include_map = BTreeMap::<String, Value>::new();
  let mut processed_includes = HashSet::<ResolvedPath>::new();

  let mut i = 0;

  while i < include_pointers_to_process.len() {
    let include_p = include_pointers_to_process[i].clone();
    i += 1;
    let mut matched = false;

    for defn in &module.definitions {
      if defn.pointer == include_p {
        matched = true;

        let ip = match ImportPattern::decode(defn) {
          Some(ip) => ip,
          None => {
            diagnostics.push(Diagnostic::internal_error(
              swc_common::DUMMY_SP,
              "Expected import pattern",
            ));

            break;
          }
        };

        if ip.kind != ImportKind::Star {
          diagnostics.push(Diagnostic::internal_error(
            swc_common::DUMMY_SP,
            "Expected import star pattern",
          ));

          break;
        }

        let path = ResolvedPath::from(ip.path);

        let inserted = processed_includes.insert(path.clone());

        if !inserted {
          break;
        }

        let matched_export_star = match included_modules.get(&path) {
          Some((_, es)) => es,
          None => {
            diagnostics.push(Diagnostic::internal_error(
              swc_common::DUMMY_SP,
              "Missing module",
            ));

            break;
          }
        };

        for (k, v) in &matched_export_star.local.properties {
          let k_string = match k {
            Value::String(k_string) => k_string.clone(),
            _ => {
              diagnostics.push(Diagnostic::internal_error(
                swc_common::DUMMY_SP,
                "Expected exported name to be a string",
              ));

              continue;
            }
          };

          let old_value = include_map.insert(k_string.clone(), v.clone());

          if old_value.is_some() {
            diagnostics.push(Diagnostic::error(
              swc_common::DUMMY_SP,
              &format!("Conflicting export {}", k_string),
            ));
          }
        }

        include_pointers_to_process.append(&mut matched_export_star.includes.clone());

        break;
      }
    }

    if !matched {
      diagnostics.push(Diagnostic::internal_error(
        swc_common::DUMMY_SP,
        "Failed to match export* pointer",
      ));
    }
  }

  let mut obj = Object::default();

  for (key, value) in include_map {
    obj.properties.push((Value::String(key), value));
  }

  obj
    .properties
    .append(&mut export_star.local.properties.clone());

  obj
}

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

#[allow(clippy::ptr_arg)]
fn calculate_content_hashes(module: &mut Module, _diagnostics: &mut Vec<Diagnostic>) {
  let ptr_to_index = module.ptr_to_index();

  let mut ptr_to_src_trace = HashMap::<Pointer, (String, Vec<Value>)>::new();
  let mut meta_to_fn = HashMap::<Pointer, Pointer>::new();

  for defn in &module.definitions {
    if let Some(src_meta) = find_ptr_src_trace(module, &ptr_to_index, &defn.pointer) {
      ptr_to_src_trace.insert(defn.pointer.clone(), src_meta);

      if let DefinitionContent::Function(fn_) = &defn.content {
        if let Some(metadata_ptr) = &fn_.metadata {
          meta_to_fn.insert(metadata_ptr.clone(), defn.pointer.clone());
        }
      };
    }
  }

  for defn in &mut module.definitions {
    match &mut defn.content {
      DefinitionContent::Function(_) => {}
      DefinitionContent::FnMeta(fn_meta) => {
        if let ContentHashable::Src(..) = &fn_meta.content_hashable {
          let content_hash =
            calculate_content_hash(&ptr_to_src_trace, meta_to_fn.get(&defn.pointer).unwrap());

          fn_meta.content_hashable = ContentHashable::Content(content_hash);
        }
      }
      DefinitionContent::Value(value) => {
        if let Value::Class(class) = value {
          let content_hash = calculate_content_hash(&ptr_to_src_trace, &defn.pointer);
          class.metadata.content_hashable = ContentHashable::Content(content_hash);
        }
      }
      DefinitionContent::Lazy(_) => {}
    }
  }
}

fn make_array_string<T, F, S>(seq: S, to_string_fn: F) -> String
where
  S: IntoIterator<Item = T>,
  F: Fn(T) -> String,
{
  format!("[{}]", comma_join(seq, to_string_fn))
}

fn comma_join<T, F, S>(seq: S, to_string_fn: F) -> String
where
  S: IntoIterator<Item = T>,
  F: Fn(T) -> String,
{
  let mut result = "".to_string();
  let mut iter = seq.into_iter();

  if let Some(first_item) = iter.next() {
    result.push_str(&to_string_fn(first_item));

    for item in iter {
      result.push(',');
      result.push_str(&to_string_fn(item));
    }
  }

  result
}

fn find_ptr_src_trace(
  module: &Module,
  ptr_to_index: &HashMap<Pointer, usize>,
  ptr: &Pointer,
) -> Option<(String, Vec<Value>)> {
  match module.get(ptr_to_index, ptr) {
    DefinitionContent::Function(fn_) => {
      let metadata_ptr = match &fn_.metadata {
        Some(ptr) => ptr,
        None => return None,
      };

      let src_meta = match module.get(ptr_to_index, metadata_ptr) {
        DefinitionContent::FnMeta(fn_meta) => match &fn_meta.content_hashable {
          ContentHashable::Src(src_hash, deps) => (Structured(src_hash).to_string(), deps.clone()),
          _ => return None,
        },
        _ => panic!("metadata_ptr did not point to metadata"),
      };

      Some(src_meta)
    }
    DefinitionContent::FnMeta(_fn_meta) => None,
    DefinitionContent::Value(value) => find_value_src_trace(module, ptr_to_index, value),
    DefinitionContent::Lazy(_) => None,
  }
}

fn find_value_src_trace(
  module: &Module,
  ptr_to_index: &HashMap<Pointer, usize>,
  value: &Value,
) -> Option<(String, Vec<Value>)> {
  match value {
    Value::Class(class) => {
      let src_meta = match &class.metadata.content_hashable {
        ContentHashable::Src(src_hash, deps) => (Structured(src_hash).to_string(), deps.clone()),
        _ => return None,
      };

      Some(src_meta)
    }
    Value::Void
    | Value::Undefined
    | Value::Null
    | Value::Bool(_)
    | Value::Number(_)
    | Value::BigInt(_)
    | Value::String(_)
    | Value::Builtin(_) => Some((Structured(value).to_string(), vec![])),
    Value::Array(array) => {
      let mut src_tags = Vec::<String>::new();
      let mut deps = Vec::<Value>::new();

      for item in &array.values {
        let (src_tag, mut item_deps) = find_value_src_trace(module, ptr_to_index, item)
          .expect("Couldn't get required source trace");

        src_tags.push(src_tag);
        deps.append(&mut item_deps);
      }

      Some((make_array_string(src_tags, |src_tag| src_tag), deps))
    }
    Value::Object(object) => {
      let mut src_tags = Vec::<String>::new();
      let mut deps = Vec::<Value>::new();

      for (k, v) in &object.properties {
        let (src_tag, mut item_deps) = find_value_src_trace(module, ptr_to_index, v)
          .expect("Couldn't get required source trace");

        src_tags.push(format!("{}:{}", Structured(k), src_tag));
        deps.append(&mut item_deps);
      }

      Some((
        format!("{{{}}}", comma_join(src_tags, |src_tag| src_tag)),
        deps,
      ))
    }
    Value::Pointer(ptr) => find_ptr_src_trace(module, ptr_to_index, ptr),
    Value::Register(_) => panic!("Can't get source trace of a register"),
  }
}

fn calculate_content_hash(
  ptr_to_src_trace: &HashMap<Pointer, (String, Vec<Value>)>,
  fn_ptr: &Pointer,
) -> Hash {
  let mut full_deps = vec![Value::Pointer(fn_ptr.clone())];
  let mut deps_included = HashSet::<Value>::new();
  deps_included.insert(Value::Pointer(fn_ptr.clone()));

  let mut i = 0;

  while i < full_deps.len() {
    let dep = full_deps[i].clone();

    let ptr_dep = match dep {
      Value::Pointer(p) => p,
      Value::Builtin(_) | Value::Undefined | Value::Number(_) => {
        i += 1;
        continue;
      }
      Value::Void
      | Value::Null
      | Value::Bool(_)
      | Value::BigInt(_)
      | Value::String(_)
      | Value::Array(_)
      | Value::Object(_)
      | Value::Class(_)
      | Value::Register(_) => {
        // undefined, Infinity, and NaN are treated as global variables, which lead them to be the
        // resolution of dependencies. All other dependencies should be builtins or pointers.
        panic!("Unexpected dependency ({})", Structured(&dep))
      }
    };

    let (_, sub_deps) = ptr_to_src_trace.get(&ptr_dep).unwrap();

    for sub_dep in sub_deps {
      if deps_included.insert(sub_dep.clone()) {
        full_deps.push(sub_dep.clone());
      }
    }

    i += 1;
  }

  let mut dep_to_index = HashMap::<Value, usize>::new();

  for (i, dep) in full_deps.iter().enumerate() {
    dep_to_index.insert(dep.clone(), i);
  }

  let mut links = Vec::<Vec<usize>>::new();

  for dep in &full_deps {
    let mut link = Vec::<usize>::new();

    match dep {
      Value::Pointer(ptr_dep) => {
        let (_, sub_deps) = ptr_to_src_trace.get(ptr_dep).unwrap();

        for sub_dep in sub_deps {
          let index = dep_to_index.get(sub_dep).unwrap();
          link.push(*index);
        }
      }
      Value::Builtin(_) | Value::Undefined | Value::Number(_) => {}
      Value::Void
      | Value::Null
      | Value::Bool(_)
      | Value::BigInt(_)
      | Value::String(_)
      | Value::Array(_)
      | Value::Object(_)
      | Value::Class(_)
      | Value::Register(_) => {
        // undefined, Infinity, and NaN are treated as global variables, which lead them to be the
        // resolution of dependencies. All other dependencies should be builtins or pointers.
        panic!("Unexpected dependency ({})", Structured(dep))
      }
    };

    links.push(link);
  }

  let mut content_trace = "{deps:".to_string();

  content_trace.push_str(&make_array_string(&full_deps, |dep| match dep {
    Value::Pointer(ptr_dep) => ptr_to_src_trace.get(ptr_dep).unwrap().0.clone(),
    Value::Builtin(_) | Value::Undefined | Value::Number(_) => Structured(dep).to_string(),
    Value::Void
    | Value::Null
    | Value::Bool(_)
    | Value::BigInt(_)
    | Value::String(_)
    | Value::Array(_)
    | Value::Object(_)
    | Value::Class(_)
    | Value::Register(_) => {
      // undefined, Infinity, and NaN are treated as global variables, which lead them to be the
      // resolution of dependencies. All other dependencies should be builtins or pointers.
      panic!("Unexpected dependency ({})", Structured(dep))
    }
  }));

  content_trace.push_str(",links:");

  content_trace.push_str(&make_array_string(&links, |link| {
    make_array_string(link, |i| i.to_string())
  }));

  content_trace.push('}');

  let mut k = Keccak::v256();
  k.update(content_trace.as_bytes());

  let mut content_hash_data = [0u8; 32];
  k.finalize(&mut content_hash_data);

  Hash(content_hash_data)
}
