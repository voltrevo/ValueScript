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

pub fn op_mod(left: Val, right: Val) -> Val {
  return Val::Number(left.to_number() % right.to_number());
}

pub fn op_exp(left: Val, right: Val) -> Val {
  return Val::Number(left.to_number().powf(right.to_number()));
}

pub fn op_eq(left: Val, right: Val) -> Val {
  if left.typeof_() != VsType::Number || right.typeof_() != VsType::Number {
    std::panic!("Not implemented");
  }

  return Val::Bool(left.to_number() == right.to_number());
}

pub fn op_ne(left: Val, right: Val) -> Val {
  if left.typeof_() != VsType::Number || right.typeof_() != VsType::Number {
    std::panic!("Not implemented");
  }

  return Val::Bool(left.to_number() != right.to_number());
}

pub fn op_triple_eq(left: Val, right: Val) -> Val {
  if left.typeof_() != VsType::Number || right.typeof_() != VsType::Number {
    std::panic!("Not implemented");
  }

  return Val::Bool(left.to_number() == right.to_number());
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

pub fn op_not(input: Val) -> Val {
  return Val::Bool(!input.is_truthy());
}

pub fn op_less(left: Val, right: Val) -> Val {
  if left.typeof_() != VsType::Number || right.typeof_() != VsType::Number {
    std::panic!("Not implemented");
  }

  return Val::Bool(left.to_number() < right.to_number());
}

pub fn op_less_eq(left: Val, right: Val) -> Val {
  if left.typeof_() != VsType::Number || right.typeof_() != VsType::Number {
    std::panic!("Not implemented");
  }

  return Val::Bool(left.to_number() <= right.to_number());
}

pub fn op_greater(left: Val, right: Val) -> Val {
  if left.typeof_() != VsType::Number || right.typeof_() != VsType::Number {
    std::panic!("Not implemented");
  }

  return Val::Bool(left.to_number() > right.to_number());
}

pub fn op_greater_eq(left: Val, right: Val) -> Val {
  if left.typeof_() != VsType::Number || right.typeof_() != VsType::Number {
    std::panic!("Not implemented");
  }

  return Val::Bool(left.to_number() >= right.to_number());
}

pub fn op_nullish_coalesce(left: Val, right: Val) -> Val {
  return if left.is_nullish() {
    right
  } else {
    left
  };
}

pub fn op_optional_chain(_left: Val, _right: Val) -> Val {
  std::panic!("Not implemented: op_optional_chain");
}

pub fn op_bit_and(_left: Val, _right: Val) -> Val {
  std::panic!("Not implemented: op_bit_and");
}

pub fn op_bit_or(_left: Val, _right: Val) -> Val {
  std::panic!("Not implemented: op_bit_or");
}

pub fn op_bit_not(_left: Val, _right: Val) -> Val {
  std::panic!("Not implemented: op_bit_not");
}

pub fn op_bit_xor(_left: Val, _right: Val) -> Val {
  std::panic!("Not implemented: op_bit_xor");
}

pub fn op_left_shift(_left: Val, _right: Val) -> Val {
  std::panic!("Not implemented: op_left_shift");
}

pub fn op_right_shift(_left: Val, _right: Val) -> Val {
  std::panic!("Not implemented: op_right_shift");
}

pub fn op_right_shift_unsigned(_left: Val, _right: Val) -> Val {
  std::panic!("Not implemented: op_right_shift_unsigned");
}

pub fn op_typeof(input: Val) -> Val {
  use VsType::*;

  return Val::String(Rc::new(match input.typeof_() {
    Undefined => "undefined".to_string(),
    Null => "object".to_string(),
    Bool => "boolean".to_string(),
    Number => "number".to_string(),
    String => "string".to_string(),
    Array => "object".to_string(),
    Object => "object".to_string(),
    Function => "function".to_string(),
  }));
}

pub fn op_instance_of(_left: Val, _right: Val) -> Val {
  std::panic!("Not implemented: op_instance_of");
}

pub fn op_in(_left: Val, _right: Val) -> Val {
  std::panic!("Not implemented: op_in");
}

pub fn op_sub(_left: Val, _right: Val) -> Val {
  std::panic!("Not implemented: op_sub");
}
