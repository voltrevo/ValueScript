use strum::EnumCount;

#[derive(strum_macros::EnumString, strum_macros::EnumCount, Clone, Copy)]
pub enum BuiltinName {
  Debug,
  Math,
  String,
  Number,
}

pub const BUILTIN_NAMES: [&str; BuiltinName::COUNT] = ["Debug", "Math", "String", "Number"];

pub const BUILTIN_COUNT: usize = BuiltinName::COUNT;

impl BuiltinName {
  pub fn to_code(&self) -> usize {
    *self as usize
  }
}
