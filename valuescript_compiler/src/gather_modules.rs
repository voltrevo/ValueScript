use std::{
  collections::{HashMap, HashSet},
  path::{Path, PathBuf},
};

use queues::{IsQueue, Queue};

use crate::{
  asm::{DefinitionContent, Instruction, InstructionOrLabel, Module, Value},
  compile, Diagnostic, DiagnosticLevel,
};

#[derive(Clone, Debug)]
enum DependencyReason {
  EntryPoint,
  ImportedBy(String),
}

#[derive(Clone)]
struct Dependency {
  path: String,
  reason: DependencyReason,
}

pub struct GatheredModules {
  pub entry_point: String,
  pub modules: HashMap<String, Module>,
  pub diagnostics: HashMap<String, Vec<Diagnostic>>,
}

pub fn gather_modules<ReadFile>(entry_point: String, read_file: ReadFile) -> GatheredModules
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

    let file_contents = match read_file(&dependency.path) {
      Ok(file_contents) => file_contents,
      Err(err) => {
        gm.diagnostics
          .entry(dependency.path.clone())
          .or_insert(vec![])
          .push(Diagnostic {
            level: DiagnosticLevel::Error,
            message: match dependency.reason {
              DependencyReason::EntryPoint => format!("File read failed: {}", err),
              DependencyReason::ImportedBy(importer) => {
                format!("Error reading file imported by {}: {}", importer, err)
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

    for imported_path in get_imported_paths(
      &compiler_output.module,
      &DependencyReason::ImportedBy(dependency.path.clone()),
    ) {
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

    gm.modules.insert(dependency.path, compiler_output.module);
  }

  gm
}

fn get_imported_paths(module: &Module, reason: &DependencyReason) -> HashSet<String> {
  let mut imported_paths = HashSet::<String>::new();

  for definition in &module.definitions {
    let lazy = match &definition.content {
      DefinitionContent::Lazy(lazy) => lazy,
      _ => continue,
    };

    match lazy.body.first() {
      Some(InstructionOrLabel::Instruction(instruction)) => {
        match instruction {
          Instruction::Import(import_path, _) | Instruction::ImportStar(import_path, _) => {
            match import_path {
              Value::String(import_path) => {
                let resolved_path = match reason {
                  DependencyReason::EntryPoint => import_path.clone(),
                  DependencyReason::ImportedBy(importer) => {
                    let importer_path = PathBuf::from(importer);
                    let parent = importer_path.parent().unwrap_or_else(|| Path::new("/"));

                    parent
                      .join(import_path)
                      .canonicalize()
                      .expect("Failed to canonicalize path")
                      .to_str()
                      .expect("Failed to convert path to string")
                      .to_string()
                  }
                };

                imported_paths.insert(resolved_path);
              }
              _ => {}
            }
          }
          _ => {}
        };
      }
      _ => {}
    }
  }

  imported_paths
}
