use crate::string_builtin::STRING_BUILTIN;

use super::debug::DEBUG;
use super::math::MATH;
use super::vs_value::ValTrait;

// TODO: Investigate whether a static array can be used for this and why rust
// seems to not like it when I try.
pub fn get_builtin(index: usize) -> &'static dyn ValTrait {
  return match index {
    0 => &MATH,
    1 => &DEBUG,
    2 => &STRING_BUILTIN,
    _ => std::panic!(""),
  };
}
