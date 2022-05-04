use std::rc::Rc;

use super::vs_value::Val;
use super::vs_value::ValTrait;
use super::vs_value::VsType;

pub fn op_plus(left: Val, right: Val) -> Val {
  let left_prim = left.to_primitive();
  let right_prim = right.to_primitive();

  if left_prim.typeof_() == VsType::String || right_prim.typeof_() == VsType::String {
    return Val::String(Rc::new(left_prim.val_to_string() + &right_prim.val_to_string()));
  }

  return Val::Number(left_prim.to_number() + right_prim.to_number());
}

pub fn op_minus(left: Val, right: Val) -> Val {
  return Val::Number(left.to_number() - right.to_number());
}

pub fn op_mul(left: Val, right: Val) -> Val {
  return Val::Number(left.to_number() * right.to_number());
}

pub fn op_div(left: Val, right: Val) -> Val {
  return Val::Number(left.to_number() / right.to_number());
}

pub fn op_exp(left: Val, right: Val) -> Val {
  return Val::Number(left.to_number().powf(right.to_number()));
}

pub fn op_mod(left: Val, right: Val) -> Val {
  return Val::Number(left.to_number() % right.to_number());
}

pub fn op_less(left: Val, right: Val) -> Val {
  if left.typeof_() != VsType::Number || right.typeof_() != VsType::Number {
    std::panic!("Not implemented");
  }

  return Val::Bool(left.to_number() < right.to_number());
}

pub fn op_triple_ne(left: Val, right: Val) -> Val {
  if left.typeof_() != VsType::Number || right.typeof_() != VsType::Number {
    std::panic!("Not implemented");
  }

  return Val::Bool(left.to_number() != right.to_number());
}

pub fn op_and(left: Val, right: Val) -> Val {
  return if left.is_truthy() {
    right
  } else {
    left
  };
}

pub fn op_or(left: Val, right: Val) -> Val {
  return if left.is_truthy() {
    left
  } else {
    right
  };
}
