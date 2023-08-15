use std::collections::{BTreeMap, HashMap, HashSet};
use std::mem::{swap, take};

use tiny_keccak::{Hasher, Keccak};

use crate::asm::{
  Builtin, ContentHashable, Definition, DefinitionContent, ExportStar, FnLine, FnMeta, Hash,
  Instruction, Object, Pointer, Value,
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
  let mut fn_to_meta = HashMap::<Pointer, Pointer>::new();
  let mut meta_to_fn = HashMap::<Pointer, Pointer>::new();
  let mut src_and_deps_map = HashMap::<Pointer, (Hash, Vec<Value>)>::new();

  for defn in &module.definitions {
    match &defn.content {
      DefinitionContent::Function(fn_) => {
        if let Some(metadata) = &fn_.metadata {
          fn_to_meta.insert(defn.pointer.clone(), metadata.clone());
          meta_to_fn.insert(metadata.clone(), defn.pointer.clone());
        }
      }
      DefinitionContent::FnMeta(fn_meta) => match &fn_meta.content_hashable {
        ContentHashable::Empty => {}
        ContentHashable::Src(src_hash, deps) => {
          src_and_deps_map.insert(defn.pointer.clone(), (src_hash.clone(), deps.clone()));
        }
        ContentHashable::Content(_) => {}
      },
      DefinitionContent::Value(_) => {}
      DefinitionContent::Lazy(_) => {}
    }
  }

  for defn in &mut module.definitions {
    match &mut defn.content {
      DefinitionContent::Function(_) => {}
      DefinitionContent::FnMeta(fn_meta) => {
        let fn_ptr = meta_to_fn.get(&defn.pointer).unwrap().clone();

        let mut full_deps = vec![PointerOrBuiltin::Pointer(fn_ptr.clone())];
        let mut deps_included = HashSet::<PointerOrBuiltin>::new();
        deps_included.insert(PointerOrBuiltin::Pointer(fn_ptr.clone()));

        let mut i = 0;

        while i < full_deps.len() {
          let dep = full_deps[i].clone();

          let ptr_dep = match dep {
            PointerOrBuiltin::Pointer(p) => p,
            PointerOrBuiltin::Builtin(_) => {
              i += 1;
              continue;
            }
          };

          let meta_ptr = fn_to_meta.get(&ptr_dep).unwrap();
          let (_, sub_deps) = src_and_deps_map.get(meta_ptr).unwrap();

          for sub_dep in sub_deps {
            match sub_dep {
              Value::Pointer(p) => {
                if deps_included.insert(PointerOrBuiltin::Pointer(p.clone())) {
                  full_deps.push(PointerOrBuiltin::Pointer(p.clone()));
                }
              }
              Value::Builtin(b) => {
                if deps_included.insert(PointerOrBuiltin::Builtin(b.clone())) {
                  full_deps.push(PointerOrBuiltin::Builtin(b.clone()));
                }
              }
              _ => {
                panic!("Expected sub_dep to be pointer or builtin")
              }
            }
          }

          i += 1;
        }

        let mut dep_to_index = HashMap::<PointerOrBuiltin, usize>::new();

        for (i, dep) in full_deps.iter().enumerate() {
          dep_to_index.insert(dep.clone(), i);
        }

        let mut links = Vec::<Vec<usize>>::new();

        for dep in &full_deps {
          let mut link = Vec::<usize>::new();

          match dep {
            PointerOrBuiltin::Pointer(ptr_dep) => {
              let meta_ptr = fn_to_meta.get(ptr_dep).unwrap();
              let (_, sub_deps) = src_and_deps_map.get(meta_ptr).unwrap();

              for sub_dep in sub_deps {
                match sub_dep {
                  Value::Pointer(p) => {
                    let index = dep_to_index
                      .get(&PointerOrBuiltin::Pointer(p.clone()))
                      .unwrap();

                    link.push(*index);
                  }
                  Value::Builtin(b) => {
                    let index = dep_to_index
                      .get(&PointerOrBuiltin::Builtin(b.clone()))
                      .unwrap();

                    link.push(*index);
                  }
                  _ => {
                    panic!("Expected sub_dep to be pointer or builtin")
                  }
                }
              }
            }
            PointerOrBuiltin::Builtin(_) => {}
          };

          links.push(link);
        }

        let mut content_trace = "{deps:".to_string();

        content_trace.push_str(&make_array_string(&full_deps, |dep| match dep {
          PointerOrBuiltin::Pointer(ptr_dep) => {
            let meta_ptr = fn_to_meta.get(ptr_dep).unwrap();
            let (src_hash, _) = src_and_deps_map.get(meta_ptr).unwrap();
            src_hash.to_string()
          }
          PointerOrBuiltin::Builtin(b) => b.to_string(),
        }));

        content_trace.push_str(",links:");

        content_trace.push_str(&make_array_string(&links, |link| {
          make_array_string(link, |i| i.to_string())
        }));

        content_trace.push('}');

        // dbg!((fn_ptr, full_deps, links, content_trace));

        let mut k = Keccak::v256();
        k.update(content_trace.as_bytes());

        let mut content_hash_data = [0u8; 32];
        k.finalize(&mut content_hash_data);

        let content_hash = Hash(content_hash_data);

        *fn_meta = FnMeta {
          name: take(&mut fn_meta.name),
          content_hashable: ContentHashable::Content(content_hash),
        };
      }
      DefinitionContent::Value(_) => {}
      DefinitionContent::Lazy(_) => {}
    }
  }
}

#[derive(Eq, Hash, PartialEq, Clone, Debug)]
enum PointerOrBuiltin {
  Pointer(Pointer),
  Builtin(Builtin),
}

fn make_array_string<T, F, S>(seq: S, to_string_fn: F) -> String
where
  S: IntoIterator<Item = T>,
  F: Fn(T) -> String,
{
  let mut result = "[".to_string();
  let mut iter = seq.into_iter();

  if let Some(first_item) = iter.next() {
    result.push_str(&to_string_fn(first_item));

    for item in iter {
      result.push(',');
      result.push_str(&to_string_fn(item));
    }
  }

  result.push(']');
  result
}
