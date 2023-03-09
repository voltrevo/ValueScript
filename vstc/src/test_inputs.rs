#[cfg(test)]
mod tests {
  use std::collections::HashSet;
  use std::fs;
  use std::path::{Path, PathBuf};

  use valuescript_compiler::compile_module;
  use valuescript_compiler::{assemble, parse_module};
  use valuescript_vm::ValTrait;
  use valuescript_vm::VirtualMachine;

  use crate::handle_diagnostics_cli::handle_diagnostics_cli;

  #[test]
  fn test_inputs() {
    let exe_path = std::env::current_exe().unwrap();
    let mut current_dir = exe_path.parent().unwrap();
    while current_dir.file_name().unwrap() != "target" {
      current_dir = current_dir.parent().unwrap();
    }
    let project_dir = current_dir.parent().unwrap(); // Go up one more level to get the project directory

    let input_dir_path = project_dir.join("inputs");

    let mut failed_paths = HashSet::<PathBuf>::new();

    let mut files = get_files_recursively(input_dir_path).expect("Failed to get files");
    files.sort();

    for file_path in files {
      let file_contents = fs::read_to_string(&file_path).expect("Failed to read file contents");

      if let Some(first_line) = file_contents.lines().next() {
        if first_line.starts_with("// test_output! ") {
          println!("\nTesting {} ...", file_path.to_str().unwrap());

          let output_str = first_line
            .split_once("// test_output! ")
            .map(|x| x.1)
            .unwrap_or("");

          let compiler_output = compile_module(&file_contents);

          handle_diagnostics_cli(
            &file_path.to_str().expect("").to_string(),
            &compiler_output.diagnostics,
          );

          for diagnostic in &compiler_output.diagnostics {
            use valuescript_compiler::DiagnosticLevel;

            match diagnostic.level {
              DiagnosticLevel::Error | DiagnosticLevel::InternalError => {
                failed_paths.insert(file_path.clone());
              }
              DiagnosticLevel::Lint | DiagnosticLevel::CompilerDebug => {}
            }
          }

          let bytecode = assemble(&compiler_output.module);

          let assembly = compiler_output.module.to_string();
          let parsed_assembly = parse_module(&assembly);
          let bytecode_via_assembly = assemble(&parsed_assembly);

          if bytecode != bytecode_via_assembly {
            println!("  Bytecode mismatch between original and parsed assembly");
            failed_paths.insert(file_path.clone());
          }

          let mut vm = VirtualMachine::new();
          let result = vm.run(&bytecode, &[]);

          let result_str = result.codify();

          if result_str != output_str {
            println!(
              "  Expected: \"{}\"\n  Actual:   \"{}\"\n",
              output_str, result_str,
            );

            failed_paths.insert(file_path.clone());
          }
        }
      }
    }

    if !failed_paths.is_empty() {
      assert!(false, "See failures above");
    }
  }

  fn get_files_recursively(dir_path: impl AsRef<Path>) -> Result<Vec<PathBuf>, std::io::Error> {
    let mut files = vec![];
    for entry in fs::read_dir(dir_path)? {
      let entry = entry?;
      let path = entry.path();
      if path.is_file() {
        files.push(path);
      } else if path.is_dir() {
        files.extend(get_files_recursively(path)?);
      }
    }
    Ok(files)
  }
}
