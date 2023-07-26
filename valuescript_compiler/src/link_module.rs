use std::collections::HashMap;

use crate::asm::{Definition, DefinitionContent, ExportStar, FnLine, Instruction, Pointer, Value};
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
  for definition in &mut module.definitions {
    let import_pattern = match ImportPattern::decode(definition) {
      Some(import_pattern) => import_pattern,
      None => continue,
    };

    let resolved_path = ResolvedPath {
      // Should have been resolved already during resolve_and_rewrite_import_patterns
      path: import_pattern.path.clone(),
    };

    let (default, namespace) = match included_modules.get(&resolved_path) {
      Some(el) => el,
      None => continue,
    };

    let new_definition = Definition {
      pointer: import_pattern.pointer,
      content: match import_pattern.kind {
        ImportKind::Default => DefinitionContent::Value(default.clone()),
        ImportKind::Star => {
          // TODO: namespace.includes
          DefinitionContent::Value(Value::Object(Box::new(namespace.local.clone())))
        }
        ImportKind::Name(name) => match namespace.local.try_resolve_key(&name) {
          Some(value) => DefinitionContent::Value(value.clone()),
          None => {
            // TODO: namespace.includes

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

    *definition = new_definition;
  }
}
