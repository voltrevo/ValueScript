use std::path::{Path, PathBuf};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ResolvedPath {
  pub path: String,
}

impl std::fmt::Display for ResolvedPath {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.path)
  }
}

pub fn resolve_path(importer_path: &ResolvedPath, path: &String) -> ResolvedPath {
  let importer_path_buf = PathBuf::from(&importer_path.path);
  let parent = importer_path_buf.parent().unwrap_or_else(|| Path::new("/"));

  ResolvedPath {
    path: parent
      .join(path)
      .canonicalize()
      .expect("Failed to canonicalize path")
      .to_str()
      .expect("Failed to convert path to string")
      .to_string(),
  }
}
