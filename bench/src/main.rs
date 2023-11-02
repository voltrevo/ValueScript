use std::{
  collections::HashSet,
  env, fs,
  path::PathBuf,
  rc::Rc,
  time::{Duration, Instant},
};

use valuescript_compiler::{assemble, compile, resolve_path, ResolvedPath};
use valuescript_vm::{vs_value::Val, Bytecode, DecoderMaker, ValTrait, VirtualMachine};

fn main() {
  let exe_path = std::env::current_exe().unwrap();
  let mut current_dir = exe_path.parent().unwrap();
  while current_dir.file_name().unwrap() != "target" {
    current_dir = current_dir.parent().unwrap();
  }
  let project_dir = current_dir.parent().unwrap(); // Go up one more level to get the project directory

  let input_dir_path = project_dir.join("inputs");

  let mut failed_paths = HashSet::<PathBuf>::new();

  let mut files = get_files_recursively(&input_dir_path).expect("Failed to get files");

  files.sort();

  let mut results = Vec::<f64>::new();

  for file_path in files {
    let file_contents = fs::read_to_string(&file_path).expect("Failed to read file contents");

    let first_line = match file_contents.lines().next() {
      Some(first_line) => first_line,
      None => continue,
    };

    if !first_line.starts_with("//! bench()") {
      continue;
    }

    let resolved_path = resolve_entry_path(file_path.to_str().expect("Failed to convert to str"));

    let compile_result = compile(resolved_path, |path| {
      fs::read_to_string(path).map_err(|err| err.to_string())
    });

    for (path, diagnostics) in compile_result.diagnostics.iter() {
      if !diagnostics.is_empty() {
        dbg!(&path.path, diagnostics);
      }

      for diagnostic in diagnostics {
        use valuescript_compiler::DiagnosticLevel;

        match diagnostic.level {
          DiagnosticLevel::Error | DiagnosticLevel::InternalError => {
            failed_paths.insert(file_path.clone());
          }
          DiagnosticLevel::Lint | DiagnosticLevel::CompilerDebug => {}
        }
      }
    }

    let friendly_file_path = file_path
      .strip_prefix(project_dir)
      .unwrap()
      .to_str()
      .unwrap();

    let module = compile_result
      .module
      .expect("Should have exited if module is None");

    let bytecode = Rc::new(Bytecode::new(assemble(&module)));

    let mut vm = VirtualMachine::default();

    let mut file_results = Vec::<f64>::new();

    let start = Instant::now();

    while Instant::now() - start < Duration::from_secs(1) {
      let before = Instant::now();
      let result = vm.run(
        None,
        &mut Val::Undefined,
        bytecode.decoder(0).decode_val(&mut vec![]),
        vec![],
      );
      let after = Instant::now();

      let duration_ms = after.duration_since(before).as_millis();

      file_results.push(duration_ms as f64);

      if let Err(result) = result {
        panic!("{} failed: {}", friendly_file_path, result.codify());
      }
    }

    let result = if file_results.len() > 2 {
      geometric_mean(&file_results[1..file_results.len() - 1])
    } else {
      geometric_mean(&file_results)
    };

    results.push(result);

    println!("{:<37} {:>6.1}ms", friendly_file_path, result);
  }

  let score = geometric_mean(&results);

  println!("{:<37} ========", "");
  println!("{:<37} {:>6.1}ms", "Score", score);

  if !failed_paths.is_empty() {
    panic!("See failures above");
  }
}

fn get_files_recursively(dir_path: &PathBuf) -> Result<Vec<PathBuf>, std::io::Error> {
  let mut files = vec![];

  for entry in fs::read_dir(dir_path)? {
    let entry = entry?;
    let path = entry.path();

    if path.is_file() {
      files.push(path);
    } else if path.is_dir() {
      files.extend(get_files_recursively(&path)?);
    }
  }

  Ok(files)
}

pub fn resolve_entry_path(entry_path: &str) -> ResolvedPath {
  // Like cwd (current working dir), but it's cwd/file.
  // This is a bit of a hack so we can use resolve_path to get the absolute path of the entry point.
  let cwd_file = ResolvedPath {
    path: env::current_dir()
      .expect("Failed to get current directory")
      .as_path()
      .join("file")
      .to_str()
      .expect("Failed to convert to str")
      .to_string(),
  };

  resolve_path(&cwd_file, entry_path)
}

fn geometric_mean(vals: &[f64]) -> f64 {
  vals.iter().product::<f64>().powf(1.0 / vals.len() as f64)
}
