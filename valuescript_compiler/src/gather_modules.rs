use std::collections::{HashMap, HashSet};

use queues::{IsQueue, Queue};

use crate::{
  asm::Module,
  compile,
  import_pattern::ImportPattern,
  resolve_path::{resolve_path, ResolvedPath},
  Diagnostic, DiagnosticLevel,
};

#[derive(Clone, Debug)]
enum DependencyReason {
  EntryPoint,
  ImportedBy(ResolvedPath),
}

#[derive(Clone)]
struct Dependency {
  path: ResolvedPath,
  reason: DependencyReason,
}

#[derive(Clone)]
pub struct PathAndModule {
  // FIXME: This should just be something like CompiledModule, and also include diagnostics
  pub path: ResolvedPath,
  pub module: Module,
}

pub struct GatheredModules {
  pub entry_point: ResolvedPath,
  pub modules: HashMap<ResolvedPath, PathAndModule>,
  pub diagnostics: HashMap<ResolvedPath, Vec<Diagnostic>>,
}

pub fn gather_modules<ReadFile>(entry_point: ResolvedPath, read_file: ReadFile) -> GatheredModules
where
  ReadFile: Fn(&str) -> Result<String, String>,
{
  let mut gm = GatheredModules {
    entry_point,
    modules: HashMap::new(),
    diagnostics: HashMap::new(),
  };

  let mut dependencies = Queue::<Dependency>::new();

  dependencies
    .add(Dependency {
      path: gm.entry_point.clone(),
      reason: DependencyReason::EntryPoint,
    })
    .expect("Failed to add to queue");

  loop {
    let dependency = match dependencies.remove() {
      Ok(dependency) => dependency,
      Err(_) => break,
    };

    let file_contents = match read_file(&dependency.path.path) {
      Ok(file_contents) => file_contents,
      Err(err) => {
        // FIXME: This diagnostic should really be attached to the import statement
        gm.diagnostics
          .entry(dependency.path.clone())
          .or_insert(vec![])
          .push(Diagnostic {
            level: DiagnosticLevel::Error,
            message: match dependency.reason {
              DependencyReason::EntryPoint => format!("File read failed: {}", err),
              DependencyReason::ImportedBy(importer) => {
                format!("File read failed: {} (imported by: {})", err, importer)
              }
            },
            span: swc_common::DUMMY_SP,
          });

        continue;
      }
    };

    let mut compiler_output = compile(&file_contents);

    gm.diagnostics
      .entry(dependency.path.clone())
      .or_insert(vec![])
      .append(&mut compiler_output.diagnostics);

    let path_and_module = PathAndModule {
      path: dependency.path.clone(),
      module: compiler_output.module,
    };

    for imported_path in get_imported_paths(&path_and_module) {
      if gm.modules.contains_key(&imported_path) {
        continue;
      }

      dependencies
        .add(Dependency {
          path: imported_path,
          reason: DependencyReason::ImportedBy(dependency.path.clone()),
        })
        .expect("Failed to add to queue");
    }

    gm.modules.insert(dependency.path, path_and_module);
  }

  gm
}

pub fn get_imported_paths(path_and_module: &PathAndModule) -> HashSet<ResolvedPath> {
  let mut imported_paths = HashSet::<ResolvedPath>::new();

  for definition in &path_and_module.module.definitions {
    let import_pattern = match ImportPattern::decode(definition) {
      Some(import_pattern) => import_pattern,
      None => continue,
    };

    imported_paths.insert(resolve_path(&path_and_module.path, &import_pattern.path));
  }

  imported_paths
}
