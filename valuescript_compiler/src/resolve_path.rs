use std::path::{Path, PathBuf};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ResolvedPath {
  pub path: String,
}

impl ResolvedPath {
  pub fn from(path: String) -> Self {
    Self { path }
  }
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
    path: normalize_path(parent.join(path))
      .to_str()
      .expect("Failed to convert path to string")
      .to_string(),
  }
}

fn normalize_path(path_buf: PathBuf) -> PathBuf {
  let mut dir_stack = Vec::new();

  for component in path_buf.components() {
    match component {
      std::path::Component::ParentDir => {
        // TODO: Error if we're at the root dir
        dir_stack.pop();
      }
      std::path::Component::CurDir => {}
      _ => {
        dir_stack.push(component);
      }
    }
  }

  let mut path_buf = PathBuf::new();
  path_buf.extend(dir_stack);

  path_buf
}
