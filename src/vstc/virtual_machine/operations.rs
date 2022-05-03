use super::vs_value::Val;
use super::vs_value::ValTrait;
use super::vs_value::VsType;
use super::vs_string::VsString;
use super::vs_number::VsNumber;
use super::vs_bool::VsBool;

pub fn op_plus(left: &Val, right: &Val) -> Val {
  let left_prim = left.to_primitive();
  let right_prim = right.to_primitive();

  if left_prim.typeof_() == VsType::String || right_prim.typeof_() == VsType::String {
    return VsString::from_string(left_prim.to_string() + &right_prim.to_string());
  }

  return VsNumber::from_f64(left_prim.to_number() + right_prim.to_number());
}

pub fn op_minus(left: &Val, right: &Val) -> Val {
  return VsNumber::from_f64(left.to_number() - right.to_number());
}

pub fn op_mul(left: &Val, right: &Val) -> Val {
  return VsNumber::from_f64(left.to_number() * right.to_number());
}

pub fn op_mod(left: &Val, right: &Val) -> Val {
  return VsNumber::from_f64(left.to_number() % right.to_number());
}

pub fn op_less(left: &Val, right: &Val) -> Val {
  if left.typeof_() != VsType::Number || right.typeof_() != VsType::Number {
    std::panic!("Not implemented");
  }

  return VsBool::from_bool(left.to_number() < right.to_number());
}

pub fn op_triple_ne(left: &Val, right: &Val) -> Val {
  if left.typeof_() != VsType::Number || right.typeof_() != VsType::Number {
    std::panic!("Not implemented");
  }

  return VsBool::from_bool(left.to_number() != right.to_number());
}
