use std::{
  collections::HashMap,
  fs,
  path::{Path, PathBuf},
};

use valuescript_compiler::{assemble, compile, resolve_path, ResolvedPath};

pub fn main() {
  let exe_path = std::env::current_exe().unwrap();
  let mut current_dir = exe_path.parent().unwrap();

  while current_dir.file_name().unwrap() != "target" {
    current_dir = current_dir.parent().unwrap();
  }

  let project_dir = current_dir.parent().unwrap(); // Go up one more level to get the project directory

  let input_dir_path = project_dir.join("inputs");

  let mut files = get_files_recursively(input_dir_path).expect("Failed to get files");
  files.sort();

  let mut file_count = 0;
  let mut total_len = 0;

  let mut bytecode_sizes = HashMap::<PathBuf, usize>::new();

  for file_path in files {
    let file_contents = fs::read_to_string(&file_path).expect("Failed to read file contents");

    if let Some(first_line) = file_contents.lines().next() {
      if !first_line.starts_with("// test_output! ") {
        continue;
      }

      let resolved_path = resolve_entry_path(
        &file_path
          .to_str()
          .expect("Failed to convert to str")
          .to_string(),
      );

      let compile_result = compile(resolved_path, |path| {
        fs::read_to_string(path).map_err(|err| err.to_string())
      });

      let module = compile_result.module.expect("Expected module");

      let bytecode = assemble(&module);
      let bytecode_len = bytecode.len();

      file_count += 1;
      total_len += bytecode_len;

      bytecode_sizes.insert(file_path, bytecode_len);
    }
  }

  println!("Compiled {} programs into {} bytes", file_count, total_len);

  let mut bytecode_sizes_vec = bytecode_sizes
    .into_iter()
    .collect::<Vec<(PathBuf, usize)>>();

  bytecode_sizes_vec.sort_by(|a, b| b.1.cmp(&a.1));

  for (file_path, bytecode_len) in bytecode_sizes_vec {
    let path = file_path.strip_prefix(project_dir).unwrap_or(&file_path);
    println!("{}: {} bytes", path.display(), bytecode_len);
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

fn resolve_entry_path(entry_path: &String) -> ResolvedPath {
  // Like cwd (current working dir), but it's cwd/file.
  // This is a bit of a hack so we can use resolve_path to get the absolute path of the entry point.
  let cwd_file = ResolvedPath {
    path: std::env::current_dir()
      .expect("Failed to get current directory")
      .as_path()
      .join("file")
      .to_str()
      .expect("Failed to convert to str")
      .to_string(),
  };

  let resolved_entry_path = resolve_path(&cwd_file, entry_path);

  resolved_entry_path
}
