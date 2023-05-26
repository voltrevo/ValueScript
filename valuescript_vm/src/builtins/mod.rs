mod array_builtin;
mod boolean_builtin;
mod builtin_object;
mod debug_builtin;
pub mod error_builtin;
mod math_builtin;
mod number_builtin;
pub mod range_error_builtin;
mod string_builtin;
mod symbol_builtin;
pub mod type_error_builtin;

use valuescript_common::BUILTIN_COUNT;

use crate::ValTrait;

pub static BUILTIN_VALS: [&'static (dyn ValTrait + Sync); BUILTIN_COUNT] = [
  &debug_builtin::DEBUG_BUILTIN,
  &math_builtin::MATH_BUILTIN,
  &string_builtin::STRING_BUILTIN,
  &number_builtin::NUMBER_BUILTIN,
  &boolean_builtin::BOOLEAN_BUILTIN,
  &array_builtin::ARRAY_BUILTIN,
  &number_builtin::IS_FINITE,
  &number_builtin::IS_NAN,
  &number_builtin::PARSE_FLOAT,
  &number_builtin::PARSE_INT,
  &error_builtin::ERROR_BUILTIN,
  &type_error_builtin::TYPE_ERROR_BUILTIN,
  &range_error_builtin::RANGE_ERROR_BUILTIN,
  &symbol_builtin::SYMBOL_BUILTIN,
];
