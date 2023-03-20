use wasm_bindgen::prelude::*;

use valuescript_compiler::{CompilerOutput, Diagnostic, DiagnosticLevel};
use valuescript_vm::ValTrait;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[derive(serde::Serialize)]
struct RunResult {
  diagnostics: Vec<valuescript_compiler::Diagnostic>,
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
      diagnostics: compiler_output.diagnostics,
      output: Err("Compile failed".into()),
    };
  }

  let bytecode = valuescript_compiler::assemble(&compiler_output.module);

  let mut vm = valuescript_vm::VirtualMachine::new();
  let result = vm.run(&bytecode, &[]);

  RunResult {
    diagnostics: compiler_output.diagnostics,
    output: match result {
      Ok(result) => Ok(result.codify()),
      Err(err) => Err(err.codify()),
    },
  }
}

#[derive(serde::Serialize)]
struct CompilerOutputWasm {
  diagnostics: Vec<Diagnostic>,
  assembly: Vec<String>,
}

impl CompilerOutputWasm {
  fn from_compiler_output(output: CompilerOutput) -> CompilerOutputWasm {
    CompilerOutputWasm {
      diagnostics: output.diagnostics,
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
