use valuescript_common::BUILTIN_COUNT;

use crate::debug_builtin::DEBUG_BUILTIN;
use crate::math_builtin::MATH_BUILTIN;
use crate::number_builtin::NUMBER_BUILTIN;
use crate::string_builtin::STRING_BUILTIN;
use crate::ValTrait;

pub static BUILTIN_VALS: [&'static (dyn ValTrait + Sync); BUILTIN_COUNT] = [
  &DEBUG_BUILTIN,
  &MATH_BUILTIN,
  &STRING_BUILTIN,
  &NUMBER_BUILTIN,
];
