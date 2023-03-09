use std::collections::HashMap;

use crate::{asm::Module, gather_modules, link_module, Diagnostic, ResolvedPath};

pub struct CompileResult {
  pub module: Option<Module>,
  pub diagnostics: HashMap<ResolvedPath, Vec<Diagnostic>>,
}

pub fn compile<ReadFile>(entry_point: ResolvedPath, read_file: ReadFile) -> CompileResult
where
  ReadFile: Fn(&str) -> Result<String, String>,
{
  let gm = gather_modules(entry_point.clone(), read_file);
  let mut link_module_result = link_module(&gm.entry_point, &gm.modules);

  let mut result = CompileResult {
    module: link_module_result.module,
    diagnostics: gm.diagnostics,
  };

  result
    .diagnostics
    .entry(entry_point)
    .or_default()
    .append(&mut link_module_result.diagnostics);

  result
}
