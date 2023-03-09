use valuescript_compiler::{resolve_path, ResolvedPath};

pub fn resolve_entry_path(entry_path: &String) -> ResolvedPath {
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
