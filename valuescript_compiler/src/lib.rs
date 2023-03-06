mod asm;
mod assembler;
mod assembly_parser;
mod capture_finder;
mod diagnostic;
mod expression_compiler;
mod function_compiler;
mod module_compiler;
mod name_allocator;
mod scope;
mod scope_analysis;

pub use assembler::assemble;
pub use assembly_parser::parse_module;
pub use diagnostic::Diagnostic;
pub use diagnostic::DiagnosticLevel;
pub use module_compiler::compile;
pub use module_compiler::CompilerOutput;
