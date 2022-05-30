use super::vs_value::ValTrait;
use super::math::MATH;
use super::debug::DEBUG;

// TODO: Investigate whether a static array can be used for this and why rust
// seems to not like it when I try.
pub fn get_builtin(index: usize) -> &'static dyn ValTrait {
  return match index {
    0 => &MATH,
    1 => &DEBUG,
    _ => std::panic!(""),
  }
}
