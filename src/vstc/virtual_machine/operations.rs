use super::vs_value::Val;
use super::vs_value::ValTrait;
use super::vs_value::VsType;
use super::vs_string::VsString;
use super::vs_number::VsNumber;

pub fn op_plus(left: &Val, right: &Val) -> Val {
  let left_prim = left.to_primitive();
  let right_prim = right.to_primitive();

  if left_prim.typeof_() == VsType::String || right_prim.typeof_() == VsType::String {
    return VsString::from_string(left_prim.to_string() + &right_prim.to_string());
  }

  return VsNumber::from_f64(left_prim.to_number() + right_prim.to_number());
}