use std::rc::Rc;

use crate::string_methods::get_string_method;

use super::vs_value::Val;
use super::vs_value::ValTrait;
use super::vs_value::VsType;

pub fn op_plus(left: Val, right: Val) -> Val {
  let left_prim = left.to_primitive();
  let right_prim = right.to_primitive();

  if left_prim.typeof_() == VsType::String || right_prim.typeof_() == VsType::String {
    return Val::String(Rc::new(
      left_prim.val_to_string() + &right_prim.val_to_string(),
    ));
  }

  return Val::Number(left_prim.to_number() + right_prim.to_number());
}

pub fn op_unary_plus(input: Val) -> Val {
  return Val::Number(input.to_number());
}

pub fn op_minus(left: Val, right: Val) -> Val {
  return Val::Number(left.to_number() - right.to_number());
}

pub fn op_unary_minus(input: Val) -> Val {
  return Val::Number(-input.to_number());
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

pub fn op_triple_eq_impl(left: Val, right: Val) -> bool {
  let type_ = left.typeof_();

  if right.typeof_() != type_ {
    return false;
  }

  match type_ {
    VsType::Undefined | VsType::Null => {
      return true;
    }
    _ => {}
  };

  return match (left, right) {
    (Val::Number(lnum), Val::Number(rnum)) => lnum == rnum,
    (Val::String(lstr), Val::String(rstr)) => lstr == rstr,
    _ => std::panic!("Not implemented"),
  };
}

pub fn op_triple_eq(left: Val, right: Val) -> Val {
  return Val::Bool(op_triple_eq_impl(left, right));
}

pub fn op_triple_ne(left: Val, right: Val) -> Val {
  return Val::Bool(!op_triple_eq_impl(left, right));
}

pub fn op_and(left: Val, right: Val) -> Val {
  return if left.is_truthy() { right } else { left };
}

pub fn op_or(left: Val, right: Val) -> Val {
  return if left.is_truthy() { left } else { right };
}

pub fn op_not(input: Val) -> Val {
  return Val::Bool(!input.is_truthy());
}

pub fn op_less(left: Val, right: Val) -> Val {
  let left_type = left.typeof_();
  let right_type = right.typeof_();

  if left_type == VsType::Undefined || right_type == VsType::Undefined {
    return Val::Bool(false);
  }

  if left_type != VsType::Number || right_type != VsType::Number {
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
  return if left.is_nullish() { right } else { left };
}

pub fn op_optional_chain(left: Val, right: Val) -> Val {
  return match left {
    Val::Undefined => Val::Undefined,
    Val::Null => Val::Undefined,

    _ => op_sub(left, right),
  };
}

fn to_i32(x: f64) -> i32 {
  if x == f64::INFINITY {
    return 0;
  }

  let int1 = (x.trunc() as i64) & 0xffffffff;

  return int1 as i32;
}

pub fn to_u32(x: f64) -> u32 {
  if x == f64::INFINITY {
    return 0;
  }

  let int1 = (x.trunc() as i64) & 0xffffffff;

  return int1 as u32;
}

pub fn op_bit_and(left: Val, right: Val) -> Val {
  let res_i32 = to_i32(left.to_number()) & to_i32(right.to_number());
  return Val::Number(res_i32 as f64);
}

pub fn op_bit_or(left: Val, right: Val) -> Val {
  let res_i32 = to_i32(left.to_number()) | to_i32(right.to_number());
  return Val::Number(res_i32 as f64);
}

pub fn op_bit_not(input: Val) -> Val {
  let res_i32 = !to_i32(input.to_number());
  return Val::Number(res_i32 as f64);
}

pub fn op_bit_xor(left: Val, right: Val) -> Val {
  let res_i32 = to_i32(left.to_number()) ^ to_i32(right.to_number());
  return Val::Number(res_i32 as f64);
}

pub fn op_left_shift(left: Val, right: Val) -> Val {
  let res_i32 = to_i32(left.to_number()) << (to_u32(right.to_number()) & 0x1f);
  return Val::Number(res_i32 as f64);
}

pub fn op_right_shift(left: Val, right: Val) -> Val {
  let res_i32 = to_i32(left.to_number()) >> (to_u32(right.to_number()) & 0x1f);
  return Val::Number(res_i32 as f64);
}

pub fn op_right_shift_unsigned(left: Val, right: Val) -> Val {
  let res_u32 = to_u32(left.to_number()) >> (to_u32(right.to_number()) & 0x1f);
  return Val::Number(res_u32 as f64);
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
    Class => "function".to_string(),
  }));
}

pub fn op_instance_of(_left: Val, _right: Val) -> Val {
  std::panic!("Not implemented: op_instance_of");
}

pub fn op_in(_left: Val, _right: Val) -> Val {
  std::panic!("Not implemented: op_in");
}

pub fn op_sub(left: Val, right: Val) -> Val {
  return match left {
    Val::Void => std::panic!("Shouldn't happen"),
    Val::Undefined => std::panic!("Not implemented: exceptions"),
    Val::Null => std::panic!("Not implemented: exceptions"),
    Val::Bool(_) => Val::Undefined,   // TODO: toString etc
    Val::Number(_) => Val::Undefined, // TODO: toString etc
    Val::String(string_data) => {
      let right_index = match right.to_index() {
        None => {
          let method = right.val_to_string();
          let method_str = method.as_str();

          return match method_str {
            "length" => Val::Number(string_data.len() as f64),
            _ => get_string_method(method_str),
          };
        }
        Some(i) => i,
      };

      let string_bytes = string_data.as_bytes();

      if right_index >= string_bytes.len() {
        return Val::Undefined;
      }

      let byte = string_bytes[right_index];

      // TODO: Val::Strings need to change to not use rust's string type,
      // because they need to represent an actual byte array underneath. This
      // occurs for invalid utf8 sequences which are getting converted to U+FFFD
      // here. To be analogous to js, the information of the actual byte needs
      // to be preserved, but that can't be represented in rust's string type.
      return Val::String(Rc::new(String::from_utf8_lossy(&[byte]).into_owned()));
    }
    Val::Array(array_data) => {
      let right_index = match right.to_index() {
        None => {
          // FIXME: Inefficient val_to_string() that gets duplicated
          // when subscripting the object
          if right.val_to_string() == "length" {
            return Val::Number(array_data.elements.len() as f64);
          }

          return array_data.object.sub(right);
        }
        Some(i) => i,
      };

      if right_index >= array_data.elements.len() {
        return Val::Undefined;
      }

      let res = array_data.elements[right_index].clone();

      return match res {
        Val::Void => Val::Undefined,
        _ => res,
      };
    }
    Val::Object(object_data) => {
      return object_data.sub(right);
    }
    Val::Function(_) => Val::Undefined,
    Val::Class(_) => Val::Undefined,
    Val::Static(s) => s.sub(right),
    Val::Custom(custom_data) => custom_data.sub(right),
  };
}

pub fn op_submov(target: &mut Val, subscript: Val, value: Val) {
  match target {
    Val::Void => std::panic!("Shouldn't happen"),
    Val::Undefined => std::panic!("Not implemented: exceptions"),
    Val::Null => std::panic!("Not implemented: exceptions"),
    Val::Bool(_) => std::panic!("Not implemented: exceptions"),
    Val::Number(_) => std::panic!("Not implemented: exceptions"),
    Val::String(_) => std::panic!("Not implemented: exceptions"),
    Val::Array(array_data) => {
      let subscript_index = match subscript.to_index() {
        None => std::panic!("Not implemented: non-uint array subscript assignment"),
        Some(i) => i,
      };

      let array_data_mut = Rc::make_mut(array_data);

      if subscript_index < array_data_mut.elements.len() {
        array_data_mut.elements[subscript_index] = value;
      } else {
        if subscript_index - array_data_mut.elements.len() > 100 {
          std::panic!("Not implemented: Sparse arrays");
        }

        while subscript_index > array_data_mut.elements.len() {
          array_data_mut.elements.push(Val::Void);
        }

        array_data_mut.elements.push(value);
      }
    }
    Val::Object(object_data) => {
      Rc::make_mut(object_data)
        .string_map
        .insert(subscript.val_to_string(), value);
    }
    Val::Function(_) => std::panic!("Not implemented: function subscript assignment"),
    Val::Class(_) => std::panic!("Not implemented: class subscript assignment"),
    Val::Static(_) => std::panic!("Not implemented: exceptions"),
    Val::Custom(_) => std::panic!("Not implemented"),
  }
}
