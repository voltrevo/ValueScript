mod vs_value;
mod vs_function;
mod vs_pointer;
mod operations;
mod bytecode_decoder;
mod virtual_machine;
mod instruction;
mod vs_object;
mod vs_array;
mod native_function;
mod builtins;
mod math;
mod vs_class;
mod plain_stack_frame;
mod stack_frame_trait;
mod first_stack_frame;

pub use virtual_machine::VirtualMachine;
