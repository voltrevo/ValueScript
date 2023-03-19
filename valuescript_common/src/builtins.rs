use strum::EnumCount;
use strum_macros::{EnumCount, EnumString};

#[derive(EnumString, EnumCount, Clone, Copy)]
pub enum BuiltinName {
  Debug,
  Math,
  String,
  Number,
  Boolean,
}

pub const BUILTIN_NAMES: [&str; BuiltinName::COUNT] =
  ["Debug", "Math", "String", "Number", "Boolean"];

pub const BUILTIN_COUNT: usize = BuiltinName::COUNT;

impl BuiltinName {
  pub fn to_code(&self) -> usize {
    *self as usize
  }
}
