mod vstc;

use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
pub fn compile(source: &str) -> String {
  let output = vstc::compile::compile(source);
  return serde_json::to_string(&output).expect("Failed json serialization");
}

#[wasm_bindgen]
pub fn run(source: &str) -> String {
  let run_result = vstc::run::run(source);
  return serde_json::to_string(&run_result).expect("Failed json serialization");
}
