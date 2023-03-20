mod array_builtin;
mod boolean_builtin;
mod debug_builtin;
mod math_builtin;
mod number_builtin;
mod string_builtin;

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
];
