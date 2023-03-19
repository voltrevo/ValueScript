use valuescript_common::BUILTIN_COUNT;

use crate::number_builtin::NUMBER_BUILTIN;
use crate::string_builtin::STRING_BUILTIN;

use super::debug::DEBUG;
use super::math::MATH;
use super::vs_value::ValTrait;

pub static BUILTIN_VALS: [&'static (dyn ValTrait + Sync); BUILTIN_COUNT] =
  [&DEBUG, &MATH, &STRING_BUILTIN, &NUMBER_BUILTIN];
