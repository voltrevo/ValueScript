use std::collections::HashMap;

use wasm_bindgen::prelude::*;

use valuescript_compiler::{
  assemble, compile as compile_internal, CompilerOutput, Diagnostic, DiagnosticLevel, ResolvedPath,
};
use valuescript_vm::ValTrait;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern "C" {
  fn js_get_error_message(e: &JsValue) -> String;
  fn js_console_log(s: &str);
}

#[derive(serde::Serialize)]
struct RunResult {
  diagnostics: HashMap<String, Vec<Diagnostic>>,
  output: Result<String, String>,
}

fn run_to_result(source: &str) -> RunResult {
  let compiler_output = valuescript_compiler::compile_module(source);

  let mut have_compiler_errors = false;

  for diagnostic in &compiler_output.diagnostics {
    match diagnostic.level {
      DiagnosticLevel::Error => have_compiler_errors = true,
      DiagnosticLevel::InternalError => have_compiler_errors = true,
      _ => (),
    }
  }

  if have_compiler_errors {
    return RunResult {
      diagnostics: vec![("(unknown)".into(), compiler_output.diagnostics)]
        .into_iter()
        .collect(),
      output: Err("Compile failed".into()),
    };
  }

  let bytecode = valuescript_compiler::assemble(&compiler_output.module);

  let mut vm = valuescript_vm::VirtualMachine::new();
  let result = vm.run(&bytecode, &[]);

  RunResult {
    diagnostics: vec![("(unknown)".into(), compiler_output.diagnostics)]
      .into_iter()
      .collect(),
    output: match result {
      Ok(result) => Ok(result.codify()),
      Err(err) => Err(err.codify()),
    },
  }
}

#[derive(serde::Serialize)]
struct CompilerOutputWasm {
  diagnostics: HashMap<String, Vec<Diagnostic>>,
  assembly: Vec<String>,
}

impl CompilerOutputWasm {
  fn from_compiler_output(output: CompilerOutput) -> CompilerOutputWasm {
    CompilerOutputWasm {
      diagnostics: vec![("(unknown)".into(), output.diagnostics)]
        .into_iter()
        .collect(),
      assembly: output.module.as_lines(),
    }
  }
}

#[wasm_bindgen]
pub fn compile(source: &str) -> String {
  let output = valuescript_compiler::compile_module(source);

  serde_json::to_string(&CompilerOutputWasm::from_compiler_output(output))
    .expect("Failed json serialization")
}

#[wasm_bindgen]
pub fn run(source: &str) -> String {
  let result = run_to_result(source);
  serde_json::to_string(&result).expect("Failed json serialization")
}

fn run_linked_to_result(entry_point: &str, read_file: &js_sys::Function) -> RunResult {
  let compile_result = compile_internal(ResolvedPath::from(entry_point.to_string()), |path| {
    let call_result = read_file.call1(&JsValue::UNDEFINED, &JsValue::from_str(path));

    match call_result {
      Ok(result) => result
        .as_string()
        .ok_or_else(|| "read_file from JS produced non-string".into()),
      Err(err) => Err(js_get_error_message(&err)),
    }
  });

  let diagnostic_len = compile_result
    .diagnostics
    .iter()
    .map(|(_, v)| v.len())
    .sum::<usize>();

  if diagnostic_len > 0 {
    return RunResult {
      diagnostics: compile_result
        .diagnostics // TODO: Avoid conversion
        .into_iter()
        .map(|(path, diagnostics)| (path.to_string(), diagnostics))
        .collect(),
      output: Err("Compile failed".into()),
    };
  }

  let module = match compile_result.module {
    Some(module) => module,
    None => {
      return RunResult {
        diagnostics: HashMap::default(),
        output: Err("Compilation did not emit module".into()),
      }
    }
  };

  let mut vm = valuescript_vm::VirtualMachine::new();

  let vm_result = vm.run(&assemble(&module), &[]);

  RunResult {
    diagnostics: HashMap::default(),
    output: match vm_result {
      Ok(result) => Ok(result.codify()),
      Err(err) => Err(err.codify()),
    },
  }
}

#[wasm_bindgen]
pub fn run_linked(entry_point: &str, read_file: &js_sys::Function) -> String {
  let result = run_linked_to_result(entry_point, read_file);
  serde_json::to_string(&result).expect("Failed json serialization")
}
