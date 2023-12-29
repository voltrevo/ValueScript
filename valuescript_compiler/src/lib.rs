pub mod asm;
mod assembler;
pub mod assembly_parser;
mod compile;
mod constants;
mod diagnostic;
mod expression_compiler;
mod function_compiler;
mod gather_modules;
mod ident;
mod import_pattern;
mod inline_valuescript;
mod instruction;
mod link_module;
mod module_compiler;
mod name_allocator;
mod optimization;
mod resolve_path;
mod scope;
mod scope_analysis;
mod src_hash;
mod static_expression_compiler;
mod target_accessor;
mod visit_pointers;

pub use assembler::assemble;
pub use assembly_parser::parse_module;
pub use compile::CompileResult;
pub use compile::{compile, compile_str};
pub use diagnostic::Diagnostic;
pub use diagnostic::DiagnosticLevel;
pub use gather_modules::gather_modules;
pub use inline_valuescript::inline_valuescript;
pub use link_module::link_module;
pub use module_compiler::compile_module;
pub use module_compiler::CompilerOutput;
pub use optimization::try_to_val::TryToVal;
pub use resolve_path::resolve_path;
pub use resolve_path::ResolvedPath;
