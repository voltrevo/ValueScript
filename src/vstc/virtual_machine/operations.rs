use super::vs_value::Val;
use super::vs_value::VsType;
use super::vs_string::VsString;
use super::vs_number::VsNumber;

pub fn op_plus(left: &Val, right: &Val) -> Val {
  if left.typeof_() == VsType::String || right.typeof_() == VsType::String {
    return VsString::from_string(left.to_string() + &right.to_string());
  }

  return VsNumber::from_f64(left.to_number() + right.to_number());
}
