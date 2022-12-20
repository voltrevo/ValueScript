mod vstc;

use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
pub fn compile(source: &str) -> String {
    return vstc::compile::full_compile_raw(source);
}

#[wasm_bindgen]
pub fn run(source: &str) -> String {
    return vstc::run::full_run_raw(source);
}