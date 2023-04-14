use wasm_bindgen::prelude::*;

use valuescript_compiler::{assemble, compile_module, Diagnostic};
use valuescript_vm::{LoadFunctionResult, ValTrait, VirtualMachine};

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[derive(serde::Serialize)]
struct RunResult {
  diagnostics: Vec<Diagnostic>,
  output: Result<String, String>,
}

fn run_to_result(entry_point: &str) -> RunResult {
  let compile_result = compile_module(entry_point);

  let diagnostic_len = compile_result.diagnostics.len();

  if diagnostic_len > 0 {
    return RunResult {
      diagnostics: compile_result.diagnostics,
      output: Err("Compile failed".into()),
    };
  }

  let module = compile_result.module;

  let bytecode = assemble(&module);

  match VirtualMachine::read_default_export(&bytecode).load_function() {
    LoadFunctionResult::NotAFunction => {
      return RunResult {
        diagnostics: vec![],
        output: Ok("(Default export is not a function)".into()),
      }
    }
    _ => {}
  };

  let mut vm = VirtualMachine::new();

  let vm_result = vm.run(&bytecode, &[]);

  RunResult {
    diagnostics: vec![],
    output: match vm_result {
      Ok(result) => Ok(result.codify()),
      Err(err) => Err(err.codify()),
    },
  }
}

#[wasm_bindgen]
pub fn run(entry_point: &str) -> String {
  let result = run_to_result(entry_point);
  serde_json::to_string(&result).expect("Failed json serialization")
}
