use std::{ffi::OsStr, fs, path::Path};

use valuescript_compiler::{assemble, compile, parse_module};
use valuescript_vm::Bytecode;

use crate::{
  handle_diagnostics_cli::handle_diagnostics_cli, resolve_entry_path::resolve_entry_path,
};

pub fn to_bytecode(format: RunFormat, file_path: &str) -> Bytecode {
  Bytecode::new(match format {
    RunFormat::TypeScript => {
      let resolved_entry_path = resolve_entry_path(file_path);

      let compile_result = compile(resolved_entry_path, |path| {
        std::fs::read_to_string(path).map_err(|err| err.to_string())
      });

      for (path, diagnostics) in compile_result.diagnostics.iter() {
        handle_diagnostics_cli(&path.path, diagnostics);
      }

      assemble(
        &compile_result
          .module
          .expect("Should have exited if module is None"),
      )
    }

    RunFormat::Assembly => {
      let file_content = std::fs::read_to_string(file_path)
        .unwrap_or_else(|_| panic!("Failed to read file {}", file_path));

      let module = parse_module(&file_content);
      assemble(&module)
    }

    RunFormat::Bytecode => {
      fs::read(file_path).unwrap_or_else(|_| panic!("Failed to read file {}", file_path))
    }
  })
}

pub enum RunFormat {
  TypeScript,
  Assembly,
  Bytecode,
}

pub fn format_from_path(file_path: &str) -> RunFormat {
  let ext = Path::new(&file_path)
    .extension()
    .and_then(OsStr::to_str)
    .unwrap_or("");

  match ext {
    "ts" | "tsx" | "mts" | "js" | "jsx" | "mjs" => RunFormat::TypeScript,
    "vsm" => RunFormat::Assembly,
    "vsb" => RunFormat::Bytecode,
    _ => std::panic!("Unrecognized file extension \"{}\"", ext),
  }
}
