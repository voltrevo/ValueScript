use std::collections::HashMap;

use wasm_bindgen::prelude::*;

use valuescript_compiler::{
  assemble, compile as compile_internal, CompileResult, Diagnostic, ResolvedPath,
};
use valuescript_vm::{LoadFunctionResult, ValTrait, VirtualMachine};

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

#[derive(serde::Serialize)]
struct CompilerOutputWasm {
  diagnostics: HashMap<String, Vec<Diagnostic>>,
  assembly: Vec<String>,
}

impl CompilerOutputWasm {
  fn from_compile_result(result: CompileResult) -> CompilerOutputWasm {
    CompilerOutputWasm {
      diagnostics: result
        .diagnostics // TODO: Avoid conversion
        .into_iter()
        .map(|(path, diagnostics)| (path.to_string(), diagnostics))
        .collect(),
      assembly: match result.module {
        Some(module) => module.as_lines(),
        None => vec![],
      },
    }
  }
}

#[wasm_bindgen]
pub fn compile(entry_point: &str, read_file: &js_sys::Function) -> String {
  let compile_result = compile_internal(ResolvedPath::from(entry_point.to_string()), |path| {
    let call_result = read_file.call1(&JsValue::UNDEFINED, &JsValue::from_str(path));

    match call_result {
      Ok(result) => result
        .as_string()
        .ok_or_else(|| "read_file from JS produced non-string".into()),
      Err(err) => Err(js_get_error_message(&err)),
    }
  });

  serde_json::to_string(&CompilerOutputWasm::from_compile_result(compile_result))
    .expect("Failed json serialization")
}

fn run_to_result(entry_point: &str, read_file: &js_sys::Function) -> RunResult {
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

  let bytecode = assemble(&module);

  match VirtualMachine::read_default_export(&bytecode).load_function() {
    LoadFunctionResult::NotAFunction => {
      return RunResult {
        diagnostics: HashMap::default(),
        output: Ok("(Default export is not a function)".into()),
      }
    }
    _ => {}
  };

  let mut vm = VirtualMachine::new();

  let vm_result = vm.run(&bytecode, &[]);

  RunResult {
    diagnostics: HashMap::default(),
    output: match vm_result {
      Ok(result) => Ok(result.codify()),
      Err(err) => Err(err.codify()),
    },
  }
}

#[wasm_bindgen]
pub fn run(entry_point: &str, read_file: &js_sys::Function) -> String {
  let result = run_to_result(entry_point, read_file);
  serde_json::to_string(&result).expect("Failed json serialization")
}
