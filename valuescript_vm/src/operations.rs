use std::mem::take;
use std::rc::Rc;
use std::str::FromStr;

use num_bigint::BigInt;
use num_bigint::Sign;
use num_traits::FromPrimitive;
use num_traits::ToPrimitive;

use crate::array_methods::op_sub_array;
use crate::bigint_methods::op_sub_bigint;
use crate::builtins::internal_error_builtin::ToInternalError;
use crate::builtins::range_error_builtin::ToRangeError;
use crate::builtins::type_error_builtin::ToTypeError;
use crate::native_function::native_fn;
use crate::native_function::NativeFunction;
use crate::number_methods::op_sub_number;
use crate::string_methods::op_sub_string;
use crate::vs_value::ToVal;
use crate::vs_value::Val;
use crate::vs_value::ValTrait;
use crate::vs_value::VsType;

pub fn op_plus(left: &Val, right: &Val) -> Result<Val, Val> {
  let left_prim = left.to_primitive();
  let right_prim = right.to_primitive();

  let left_type = left_prim.typeof_();
  let right_type = right_prim.typeof_();

  if left_type == VsType::String || right_type == VsType::String {
    return Ok((left_prim.to_string() + &right_prim.to_string()).to_val());
  }

  if left_type == VsType::BigInt || right_type == VsType::BigInt {
    if left_type != right_type {
      return Err("Cannot mix BigInt and other types".to_type_error());
    }

    match (left_prim.as_bigint_data(), right_prim.as_bigint_data()) {
      (Some(left_bigint), Some(right_bigint)) => {
        return Ok(Val::BigInt(left_bigint + right_bigint));
      }
      _ => return Err("TODO: plus with bigint and non-bigint".to_internal_error()),
    }
  }

  return Ok(Val::Number(left_prim.to_number() + right_prim.to_number()));
}

pub fn op_unary_plus(input: &Val) -> Val {
  match input.as_bigint_data() {
    Some(bigint) => Val::BigInt(bigint),
    _ => Val::Number(input.to_number()),
  }
}

pub fn op_minus(left: &Val, right: &Val) -> Result<Val, Val> {
  match (left.as_bigint_data(), right.as_bigint_data()) {
    (Some(left_bigint), Some(right_bigint)) => Ok(Val::BigInt(left_bigint - right_bigint)),
    (Some(_), None) | (None, Some(_)) => {
      return Err("Cannot mix BigInt with other types".to_type_error())
    }
    _ => Ok(Val::Number(left.to_number() - right.to_number())),
  }
}

pub fn op_unary_minus(input: &Val) -> Val {
  match input.as_bigint_data() {
    Some(bigint) => Val::BigInt(-bigint),
    _ => Val::Number(-input.to_number()),
  }
}

pub fn op_mul(left: &Val, right: &Val) -> Result<Val, Val> {
  match (left.as_bigint_data(), right.as_bigint_data()) {
    (Some(left_bigint), Some(right_bigint)) => Ok(Val::BigInt(left_bigint * right_bigint)),
    (Some(_), None) | (None, Some(_)) => Err("Cannot mix BigInt with other types".to_type_error()),
    _ => Ok(Val::Number(left.to_number() * right.to_number())),
  }
}

pub fn op_div(left: &Val, right: &Val) -> Result<Val, Val> {
  match (left.as_bigint_data(), right.as_bigint_data()) {
    (Some(left_bigint), Some(right_bigint)) => Ok(Val::BigInt(left_bigint / right_bigint)),
    (Some(_), None) | (None, Some(_)) => Err("Cannot mix BigInt with other types".to_type_error()),
    _ => Ok(Val::Number(left.to_number() / right.to_number())),
  }
}

pub fn op_mod(left: &Val, right: &Val) -> Result<Val, Val> {
  match (left.as_bigint_data(), right.as_bigint_data()) {
    (Some(left_bigint), Some(right_bigint)) => Ok(Val::BigInt(left_bigint % right_bigint)),
    (Some(_), None) | (None, Some(_)) => Err("Cannot mix BigInt with other types".to_type_error()),
    _ => Ok(Val::Number(left.to_number() % right.to_number())),
  }
}

pub fn op_exp(left: &Val, right: &Val) -> Result<Val, Val> {
  match (left.as_bigint_data(), right.as_bigint_data()) {
    (Some(left_bigint), Some(right_bigint)) => {
      if right_bigint.sign() == Sign::Minus {
        return Err("Exponent must be non-negative".to_range_error());
      }

      let exp = match right_bigint.to_u32() {
        Some(exp) => exp,
        None => return Err("Exponent must be less than 2^32".to_range_error()),
      };

      Ok(Val::BigInt(left_bigint.pow(exp)))
    }
    (Some(_), None) | (None, Some(_)) => Err("Cannot mix BigInt with other types".to_type_error()),
    _ => Ok(Val::Number(left.to_number().powf(right.to_number()))),
  }
}

pub fn op_eq_impl(left: &Val, right: &Val) -> Result<bool, Val> {
  Ok(match (left, right) {
    (left, Val::Undefined | Val::Null) => match left {
      Val::Undefined | Val::Null => true,
      _ => false,
    },
    (Val::Bool(left_bool), Val::Bool(right_bool)) => left_bool == right_bool,
    (Val::Number(left_number), Val::Number(right_number)) => left_number == right_number,
    (Val::String(left_string), Val::String(right_string)) => left_string == right_string,
    (Val::BigInt(left_bigint), Val::BigInt(right_bigint)) => left_bigint == right_bigint,
    _ => {
      return Err(
        format!(
          "TODO: op== with other types ({}, {})",
          left.codify(),
          right.codify()
        )
        .to_internal_error(),
      )
    }
  })
}

pub fn op_eq(left: &Val, right: &Val) -> Result<Val, Val> {
  Ok(Val::Bool(op_eq_impl(left, right)?))
}

pub fn op_ne(left: &Val, right: &Val) -> Result<Val, Val> {
  Ok(Val::Bool(!op_eq_impl(left, right)?))
}

pub fn op_triple_eq_impl(left: &Val, right: &Val) -> Result<bool, Val> {
  Ok(match (left, right) {
    (Val::Undefined, Val::Undefined) => true,
    (Val::Null, Val::Null) => true,
    (Val::Bool(left_bool), Val::Bool(right_bool)) => left_bool == right_bool,
    (Val::Number(left_number), Val::Number(right_number)) => left_number == right_number,
    (Val::String(left_string), Val::String(right_string)) => left_string == right_string,
    (Val::BigInt(left_bigint), Val::BigInt(right_bigint)) => left_bigint == right_bigint,
    _ => {
      if left.typeof_() != right.typeof_() {
        false
      } else {
        return Err("TODO: op=== with other types".to_internal_error());
      }
    }
  })
}

pub fn op_triple_eq(left: &Val, right: &Val) -> Result<Val, Val> {
  let is_eq = op_triple_eq_impl(left, right)?;
  Ok(Val::Bool(is_eq))
}

pub fn op_triple_ne(left: &Val, right: &Val) -> Result<Val, Val> {
  let is_eq = op_triple_eq_impl(left, right)?;
  Ok(Val::Bool(!is_eq))
}

pub fn op_and(left: &Val, right: &Val) -> Result<Val, Val> {
  let truthy = left.is_truthy();

  Ok((if truthy { right } else { left }).clone())
}

pub fn op_or(left: &Val, right: &Val) -> Result<Val, Val> {
  let truthy = left.is_truthy();

  Ok((if truthy { left } else { right }).clone())
}

pub fn op_not(input: &Val) -> Val {
  return Val::Bool(!input.is_truthy());
}

pub fn op_less(left: &Val, right: &Val) -> Result<Val, Val> {
  Ok(Val::Bool(match (left, right) {
    (_, Val::Undefined) | (Val::Undefined, _) => false,
    (Val::Null, Val::Null) => false,
    (Val::Bool(left_bool), Val::Bool(right_bool)) => left_bool < right_bool,
    (Val::Number(left_number), Val::Number(right_number)) => left_number < right_number,
    (Val::String(left_string), Val::String(right_string)) => left_string < right_string,
    (Val::BigInt(left_bigint), Val::BigInt(right_bigint)) => left_bigint < right_bigint,
    _ => match ecma_is_less_than(left, right) {
      None => false,
      Some(x) => x,
    },
  }))
}

pub fn op_less_eq(left: &Val, right: &Val) -> Result<Val, Val> {
  Ok(Val::Bool(match (left, right) {
    (_, Val::Undefined) | (Val::Undefined, _) => false,
    (Val::Null, Val::Null) => true,
    (Val::Bool(left_bool), Val::Bool(right_bool)) => left_bool <= right_bool,
    (Val::Number(left_number), Val::Number(right_number)) => left_number <= right_number,
    (Val::String(left_string), Val::String(right_string)) => left_string <= right_string,
    (Val::BigInt(left_bigint), Val::BigInt(right_bigint)) => left_bigint <= right_bigint,
    _ => match ecma_is_less_than(right, left) {
      None => false,
      Some(x) => !x,
    },
  }))
}

pub fn op_greater(left: &Val, right: &Val) -> Result<Val, Val> {
  Ok(Val::Bool(match (left, right) {
    (_, Val::Undefined) | (Val::Undefined, _) => false,
    (Val::Null, Val::Null) => false,
    (Val::Bool(left_bool), Val::Bool(right_bool)) => left_bool > right_bool,
    (Val::Number(left_number), Val::Number(right_number)) => left_number > right_number,
    (Val::String(left_string), Val::String(right_string)) => left_string > right_string,
    (Val::BigInt(left_bigint), Val::BigInt(right_bigint)) => left_bigint > right_bigint,
    _ => match ecma_is_less_than(right, left) {
      None => false,
      Some(x) => x,
    },
  }))
}

pub fn op_greater_eq(left: &Val, right: &Val) -> Result<Val, Val> {
  Ok(Val::Bool(match (left, right) {
    (_, Val::Undefined) | (Val::Undefined, _) => false,
    (Val::Null, Val::Null) => true,
    (Val::Bool(left_bool), Val::Bool(right_bool)) => left_bool >= right_bool,
    (Val::Number(left_number), Val::Number(right_number)) => left_number >= right_number,
    (Val::String(left_string), Val::String(right_string)) => left_string >= right_string,
    (Val::BigInt(left_bigint), Val::BigInt(right_bigint)) => left_bigint >= right_bigint,
    (left, right) => match ecma_is_less_than(left, right) {
      None => false,
      Some(x) => !x,
    },
  }))
}

pub fn op_nullish_coalesce(left: &Val, right: &Val) -> Result<Val, Val> {
  let nullish = left.is_nullish();

  Ok((if nullish { right } else { left }).clone())
}

pub fn op_optional_chain(left: &mut Val, right: &Val) -> Result<Val, Val> {
  return match left {
    Val::Undefined | Val::Null => Ok(Val::Undefined),

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

pub fn op_bit_and(left: &Val, right: &Val) -> Result<Val, Val> {
  match (left.as_bigint_data(), right.as_bigint_data()) {
    (Some(left_bigint), Some(right_bigint)) => Ok(Val::BigInt(left_bigint & right_bigint)),
    (Some(_), None) | (None, Some(_)) => Err("Cannot mix BigInt with other types".to_type_error()),
    _ => {
      let res_i32 = to_i32(left.to_number()) & to_i32(right.to_number());
      Ok(Val::Number(res_i32 as f64))
    }
  }
}

pub fn op_bit_or(left: &Val, right: &Val) -> Result<Val, Val> {
  match (left.as_bigint_data(), right.as_bigint_data()) {
    (Some(left_bigint), Some(right_bigint)) => Ok(Val::BigInt(left_bigint | right_bigint)),
    (Some(_), None) | (None, Some(_)) => Err("Cannot mix BigInt with other types".to_type_error()),
    _ => {
      let res_i32 = to_i32(left.to_number()) | to_i32(right.to_number());
      Ok(Val::Number(res_i32 as f64))
    }
  }
}

pub fn op_bit_not(input: &Val) -> Val {
  match input.as_bigint_data() {
    Some(bigint) => Val::BigInt(!bigint),
    None => {
      let res_i32 = !to_i32(input.to_number());
      Val::Number(res_i32 as f64)
    }
  }
}

pub fn op_bit_xor(left: &Val, right: &Val) -> Result<Val, Val> {
  match (left.as_bigint_data(), right.as_bigint_data()) {
    (Some(left_bigint), Some(right_bigint)) => Ok(Val::BigInt(left_bigint ^ right_bigint)),
    (Some(_), None) | (None, Some(_)) => Err("Cannot mix BigInt with other types".to_type_error()),
    _ => {
      let res_i32 = to_i32(left.to_number()) ^ to_i32(right.to_number());
      Ok(Val::Number(res_i32 as f64))
    }
  }
}

pub fn op_left_shift(left: &Val, right: &Val) -> Result<Val, Val> {
  match (left.as_bigint_data(), right.as_bigint_data()) {
    (Some(left_bigint), Some(right_bigint)) => Ok(Val::BigInt(
      left_bigint << right_bigint.to_i64().expect("TODO"),
    )),
    (Some(_), None) | (None, Some(_)) => Err("Cannot mix BigInt with other types".to_type_error()),
    _ => {
      let res_i32 = to_i32(left.to_number()) << (to_u32(right.to_number()) & 0x1f);
      Ok(Val::Number(res_i32 as f64))
    }
  }
}

pub fn op_right_shift(left: &Val, right: &Val) -> Result<Val, Val> {
  match (left.as_bigint_data(), right.as_bigint_data()) {
    (Some(left_bigint), Some(right_bigint)) => {
      let right_i64 = right_bigint
        .to_i64()
        .ok_or("TODO: handle i64 conversion failure".to_val())?;
      Ok(Val::BigInt(left_bigint >> right_i64))
    }
    (Some(_), None) | (None, Some(_)) => Err("Cannot mix BigInt with other types".to_type_error()),
    _ => {
      let res_i32 = to_i32(left.to_number()) >> (to_u32(right.to_number()) & 0x1f);
      Ok(Val::Number(res_i32 as f64))
    }
  }
}

pub fn op_right_shift_unsigned(left: &Val, right: &Val) -> Result<Val, Val> {
  match (left.as_bigint_data(), right.as_bigint_data()) {
    (Some(_), Some(_)) => Err("BigInts don't support unsigned right shift".to_type_error()),
    (Some(_), None) | (None, Some(_)) => Err("Cannot mix BigInt with other types".to_type_error()),
    _ => {
      let res_u32 = to_u32(left.to_number()) >> (to_u32(right.to_number()) & 0x1f);
      Ok(Val::Number(res_u32 as f64))
    }
  }
}

pub fn op_typeof(input: &Val) -> Val {
  use VsType::*;

  match input.typeof_() {
    Undefined => "undefined",
    Null => "object",
    Bool => "boolean",
    Number => "number",
    BigInt => "bigint",
    Symbol => "symbol",
    String => "string",
    Array => "object",
    Object => "object",
    Function => "function",
    Class => "function",
  }
  .to_val()
}

pub fn op_instance_of(_left: &Val, _right: &Val) -> Result<Val, Val> {
  Err("TODO: op_instance_of".to_internal_error())
}

pub fn op_in(_left: &Val, _right: &Val) -> Result<Val, Val> {
  Err("TODO: op_in".to_internal_error())
}

pub fn op_sub(left: &mut Val, right: &Val) -> Result<Val, Val> {
  match left {
    Val::Void => Err("Internal: Shouldn't happen".to_internal_error()), // TODO: Internal errors
    Val::Undefined => Err("Cannot subscript undefined".to_type_error()),
    Val::Null => Err("Cannot subscript null".to_type_error()),
    Val::Bool(_) => Ok(match right.to_string().as_str() {
      "toString" => BOOL_TO_STRING.to_val(),
      "valueOf" => BOOL_VALUE_OF.to_val(),
      _ => Val::Undefined,
    }),
    Val::Number(number) => Ok(op_sub_number(*number, right)),
    Val::BigInt(bigint) => Ok(op_sub_bigint(bigint, right)),
    Val::Symbol(_) => Ok(Val::Undefined),
    Val::String(string_data) => Ok(op_sub_string(string_data, right)),
    Val::Array(array_data) => {
      let right_index = match right.to_index() {
        None => {
          // FIXME: Inefficient to_string() that gets duplicated
          // when subscripting the object
          if right.to_string() == "length" {
            return Ok(Val::Number(array_data.elements.len() as f64));
          }

          return op_sub_array(array_data, right);
        }
        Some(i) => i,
      };

      if right_index >= array_data.elements.len() {
        return Ok(Val::Undefined);
      }

      let res = match Rc::get_mut(array_data) {
        Some(array_data) => take(&mut array_data.elements[right_index]),
        None => array_data.elements[right_index].clone(),
      };

      return Ok(match res {
        Val::Void => Val::Undefined,
        _ => res,
      });
    }
    Val::Object(object_data) => Ok(object_data.sub(right)), // TODO: move on single ref
    Val::Function(_) => Ok(Val::Undefined),
    Val::Class(class) => op_sub(&mut class.static_.clone(), right),
    Val::Static(s) => s.sub(right),
    Val::Dynamic(dynamic_data) => dynamic_data.sub(right),
    Val::CopyCounter(cc) => Ok(match right.to_string().as_str() {
      "tag" => cc.tag.clone(),
      "count" => (*cc.count.borrow() as f64).to_val(),
      _ => Val::Undefined,
    }),
  }
}

pub fn op_submov(target: &mut Val, subscript: &Val, value: Val) -> Result<(), Val> {
  match target {
    Val::Void => Err("Internal: Shouldn't happen".to_internal_error()), // TODO: Internal errors
    Val::Undefined => Err("Cannot assign to subscript of undefined".to_type_error()),
    Val::Null => Err("Cannot assign to subscript of null".to_type_error()),
    Val::Bool(_) => Err("Cannot assign to subscript of bool".to_type_error()),
    Val::Number(_) => Err("Cannot assign to subscript of number".to_type_error()),
    Val::BigInt(_) => Err("Cannot assign to subscript of bigint".to_type_error()),
    Val::Symbol(_) => Err("Cannot assign to subscript of symbol".to_type_error()),
    Val::String(_) => Err("Cannot assign to subscript of string".to_type_error()),
    Val::Array(array_data) => {
      let subscript_index = match subscript.to_index() {
        // TODO: Internal errors
        None => return Err("TODO: non-uint array subscript assignment".to_type_error()),
        Some(i) => i,
      };

      let array_data_mut = Rc::make_mut(array_data);

      if subscript_index < array_data_mut.elements.len() {
        array_data_mut.elements[subscript_index] = value;
      } else {
        if subscript_index - array_data_mut.elements.len() > 100 {
          return Err("TODO: Sparse arrays".to_type_error());
        }

        while subscript_index > array_data_mut.elements.len() {
          array_data_mut.elements.push(Val::Void);
        }

        array_data_mut.elements.push(value);
      }

      Ok(())
    }
    Val::Object(object_data) => {
      let object_data_mut = Rc::make_mut(object_data);

      match subscript {
        Val::String(string) => object_data_mut.string_map.insert(string.to_string(), value),
        Val::Symbol(symbol) => object_data_mut.symbol_map.insert(symbol.clone(), value),
        _ => object_data_mut
          .string_map
          .insert(subscript.to_string(), value),
      };

      Ok(())
    }
    Val::Function(_) => Err("TODO: function subscript assignment".to_type_error()),
    Val::Class(_) => Err("Cannot assign to subscript of class".to_type_error()),
    Val::Static(_) => Err("Cannot assign to subscript of static value".to_type_error()),
    Val::Dynamic(_) => Err("TODO: Assign to subscript of dynamic value".to_type_error()),
    Val::CopyCounter(_) => Err("Cannot assign to subscript of CopyCounter".to_type_error()),
  }
}

pub fn ecma_is_less_than(x: &Val, y: &Val) -> Option<bool> {
  let px = x.to_primitive();
  let py = y.to_primitive();

  match (px, py) {
    (Val::BigInt(x), Val::BigInt(y)) => Some(x < y),
    (Val::String(x), Val::String(y)) => Some(x < y),
    (Val::BigInt(x), Val::String(y)) => match BigInt::from_str(&y) {
      Ok(y) => Some(x < y),
      Err(_) => None,
    },
    (Val::String(x), Val::BigInt(y)) => match BigInt::from_str(&x) {
      Ok(x) => Some(x < y),
      Err(_) => None,
    },
    (Val::BigInt(x), y) => {
      let y = y.to_number();

      if y.is_nan() {
        return None;
      }

      if y.abs() == f64::INFINITY {
        return Some(y.is_sign_positive());
      }

      let y_floor = y.floor();
      let y_floor_big = BigInt::from_f64(y_floor).unwrap();

      if x < y_floor_big {
        return Some(false);
      }

      if x == y_floor_big {
        return Some(y != y_floor);
      }

      Some(false)
    }
    (x, Val::BigInt(y)) => {
      let x = x.to_number();

      if x.is_nan() {
        return None;
      }

      if x.abs() == f64::INFINITY {
        return Some(x.is_sign_negative());
      }

      let x_ceil = x.ceil();
      let x_ceil_big = BigInt::from_f64(x_ceil).unwrap();

      if x_ceil_big < y {
        return Some(true);
      }

      if x_ceil_big == y {
        return Some(x != x_ceil);
      }

      Some(false)
    }
    (x, y) => {
      let x = x.to_number();
      let y = y.to_number();

      if x.is_nan() || y.is_nan() {
        None
      } else {
        Some(x < y)
      }
    }
  }
}

static BOOL_TO_STRING: NativeFunction = native_fn(|this, _params| {
  Ok(match this.get() {
    Val::Bool(b) => b.to_string().to_val(),
    _ => return Err("bool indirection".to_type_error()),
  })
});

static BOOL_VALUE_OF: NativeFunction = native_fn(|this, _params| {
  Ok(match this.get() {
    Val::Bool(b) => Val::Bool(*b),
    _ => return Err("bool indirection".to_type_error()),
  })
});
