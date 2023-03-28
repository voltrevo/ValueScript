mod asm;
mod assembler;
mod assembly_parser;
mod compile;
mod constants;
mod diagnostic;
mod expression_compiler;
mod function_compiler;
mod gather_modules;
mod import_pattern;
mod link_module;
mod module_compiler;
mod name_allocator;
mod resolve_path;
mod scope_analysis;

pub use assembler::assemble;
pub use assembly_parser::parse_module;
pub use compile::compile;
pub use compile::CompileResult;
pub use diagnostic::Diagnostic;
pub use diagnostic::DiagnosticLevel;
pub use gather_modules::gather_modules;
pub use link_module::link_module;
pub use module_compiler::compile_module;
pub use module_compiler::CompilerOutput;
pub use resolve_path::resolve_path;
pub use resolve_path::ResolvedPath;
