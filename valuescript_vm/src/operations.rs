use std::rc::Rc;

use num_bigint::Sign;
use num_traits::ToPrimitive;

use crate::bigint_methods::op_sub_bigint;
use crate::native_function::NativeFunction;
use crate::number_methods::op_sub_number;
use crate::string_methods::op_sub_string;

use super::vs_value::Val;
use super::vs_value::ValTrait;
use super::vs_value::VsType;

macro_rules! format_err {
  ($fmt:expr $(, $($arg:expr),*)?) => {{
      let formatted_string = format!($fmt $(, $($arg),*)?);

      // TODO: This should be a proper error type
      Err(Val::String(Rc::new(formatted_string)))
  }};
}

macro_rules! format_string {
  ($fmt:expr $(, $($arg:expr),*)?) => {{
      let formatted_string = format!($fmt $(, $($arg),*)?);
      Val::String(Rc::new(formatted_string))
  }};
}

pub fn op_plus(left: Val, right: Val) -> Result<Val, Val> {
  let left_prim = left.to_primitive();
  let right_prim = right.to_primitive();

  let left_type = left_prim.typeof_();
  let right_type = right_prim.typeof_();

  if left_type == VsType::String || right_type == VsType::String {
    return Ok(Val::String(Rc::new(
      left_prim.val_to_string() + &right_prim.val_to_string(),
    )));
  }

  if left_type == VsType::BigInt || right_type == VsType::BigInt {
    if left_type != right_type {
      return Err(Val::String(Rc::new(
        "TypeError: Cannot mix BigInt and other types".to_string(),
      )));
    }

    match (left_prim.as_bigint_data(), right_prim.as_bigint_data()) {
      (Some(left_bigint), Some(right_bigint)) => {
        return Ok(Val::BigInt(left_bigint + right_bigint));
      }
      _ => return format_err!("TODO"),
    }
  }

  return Ok(Val::Number(left_prim.to_number() + right_prim.to_number()));
}

pub fn op_unary_plus(input: Val) -> Val {
  match input.as_bigint_data() {
    Some(bigint) => Val::BigInt(bigint),
    _ => Val::Number(input.to_number()),
  }
}

pub fn op_minus(left: Val, right: Val) -> Result<Val, Val> {
  match (left.as_bigint_data(), right.as_bigint_data()) {
    (Some(left_bigint), Some(right_bigint)) => Ok(Val::BigInt(left_bigint - right_bigint)),
    (Some(_), None) | (None, Some(_)) => Err(Val::String(Rc::new(
      "TypeError: Cannot mix BigInt with other types".to_string(),
    ))),
    _ => Ok(Val::Number(left.to_number() - right.to_number())),
  }
}

pub fn op_unary_minus(input: Val) -> Val {
  match input.as_bigint_data() {
    Some(bigint) => Val::BigInt(-bigint),
    _ => Val::Number(-input.to_number()),
  }
}

pub fn op_mul(left: Val, right: Val) -> Result<Val, Val> {
  match (left.as_bigint_data(), right.as_bigint_data()) {
    (Some(left_bigint), Some(right_bigint)) => Ok(Val::BigInt(left_bigint * right_bigint)),
    (Some(_), None) | (None, Some(_)) => {
      format_err!("TypeError: Cannot mix BigInt with other types")
    }
    _ => Ok(Val::Number(left.to_number() * right.to_number())),
  }
}

pub fn op_div(left: Val, right: Val) -> Result<Val, Val> {
  match (left.as_bigint_data(), right.as_bigint_data()) {
    (Some(left_bigint), Some(right_bigint)) => Ok(Val::BigInt(left_bigint / right_bigint)),
    (Some(_), None) | (None, Some(_)) => {
      format_err!("TypeError: Cannot mix BigInt with other types")
    }
    _ => Ok(Val::Number(left.to_number() / right.to_number())),
  }
}

pub fn op_mod(left: Val, right: Val) -> Result<Val, Val> {
  match (left.as_bigint_data(), right.as_bigint_data()) {
    (Some(left_bigint), Some(right_bigint)) => Ok(Val::BigInt(left_bigint % right_bigint)),
    (Some(_), None) | (None, Some(_)) => {
      format_err!("TypeError: Cannot mix BigInt with other types")
    }
    _ => Ok(Val::Number(left.to_number() % right.to_number())),
  }
}

pub fn op_exp(left: Val, right: Val) -> Result<Val, Val> {
  match (left.as_bigint_data(), right.as_bigint_data()) {
    (Some(left_bigint), Some(right_bigint)) => {
      if right_bigint.sign() == Sign::Minus {
        return format_err!("RangeError: Exponent must be non-negative");
      }

      let exp = match right_bigint.to_u32() {
        Some(exp) => exp,
        None => return format_err!("RangeError: Exponent must be less than 2^32"),
      };

      Ok(Val::BigInt(left_bigint.pow(exp)))
    }
    (Some(_), None) | (None, Some(_)) => {
      format_err!("TypeError: Cannot mix BigInt with other types")
    }
    _ => Ok(Val::Number(left.to_number().powf(right.to_number()))),
  }
}

pub fn op_eq(left: Val, right: Val) -> Result<Val, Val> {
  Ok(Val::Bool(match (left, right) {
    (Val::Undefined, Val::Undefined) => true,
    (Val::Null, Val::Null) => true,
    (Val::Bool(left_bool), Val::Bool(right_bool)) => left_bool == right_bool,
    (Val::Number(left_number), Val::Number(right_number)) => left_number == right_number,
    (Val::String(left_string), Val::String(right_string)) => left_string == right_string,
    (Val::BigInt(left_bigint), Val::BigInt(right_bigint)) => left_bigint == right_bigint,
    _ => return format_err!("TODO"),
  }))
}

pub fn op_ne(left: Val, right: Val) -> Result<Val, Val> {
  Ok(Val::Bool(match (left, right) {
    (Val::Undefined, Val::Undefined) => false,
    (Val::Null, Val::Null) => false,
    (Val::Bool(left_bool), Val::Bool(right_bool)) => left_bool != right_bool,
    (Val::Number(left_number), Val::Number(right_number)) => left_number != right_number,
    (Val::String(left_string), Val::String(right_string)) => left_string != right_string,
    (Val::BigInt(left_bigint), Val::BigInt(right_bigint)) => left_bigint != right_bigint,
    _ => return format_err!("TODO"),
  }))
}

pub fn op_triple_eq_impl(left: Val, right: Val) -> Result<bool, Val> {
  Ok(match (&left, &right) {
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
        return format_err!("TODO");
      }
    }
  })
}

pub fn op_triple_eq(left: Val, right: Val) -> Result<Val, Val> {
  let is_eq = op_triple_eq_impl(left, right)?;
  Ok(Val::Bool(is_eq))
}

pub fn op_triple_ne(left: Val, right: Val) -> Result<Val, Val> {
  let is_eq = op_triple_eq_impl(left, right)?;
  Ok(Val::Bool(!is_eq))
}

pub fn op_and(left: Val, right: Val) -> Result<Val, Val> {
  Ok(if left.is_truthy() { right } else { left })
}

pub fn op_or(left: Val, right: Val) -> Result<Val, Val> {
  Ok(if left.is_truthy() { left } else { right })
}

pub fn op_not(input: Val) -> Val {
  return Val::Bool(!input.is_truthy());
}

pub fn op_less(left: Val, right: Val) -> Result<Val, Val> {
  Ok(Val::Bool(match (&left, &right) {
    (Val::Undefined, Val::Undefined) => false,
    (Val::Null, Val::Null) => false,
    (Val::Bool(left_bool), Val::Bool(right_bool)) => left_bool < right_bool,
    (Val::Number(left_number), Val::Number(right_number)) => left_number < right_number,
    (Val::String(left_string), Val::String(right_string)) => left_string < right_string,
    (Val::BigInt(left_bigint), Val::BigInt(right_bigint)) => left_bigint < right_bigint,
    _ => {
      if left.typeof_() == VsType::Undefined || right.typeof_() == VsType::Undefined {
        false
      } else {
        return format_err!("TODO");
      }
    }
  }))
}

pub fn op_less_eq(left: Val, right: Val) -> Result<Val, Val> {
  Ok(Val::Bool(match (left, right) {
    (Val::Undefined, Val::Undefined) => false,
    (Val::Null, Val::Null) => true,
    (Val::Bool(left_bool), Val::Bool(right_bool)) => left_bool <= right_bool,
    (Val::Number(left_number), Val::Number(right_number)) => left_number <= right_number,
    (Val::String(left_string), Val::String(right_string)) => left_string <= right_string,
    (Val::BigInt(left_bigint), Val::BigInt(right_bigint)) => left_bigint <= right_bigint,
    _ => return format_err!("TODO"),
  }))
}

pub fn op_greater(left: Val, right: Val) -> Result<Val, Val> {
  Ok(Val::Bool(match (left, right) {
    (Val::Undefined, Val::Undefined) => false,
    (Val::Null, Val::Null) => false,
    (Val::Bool(left_bool), Val::Bool(right_bool)) => left_bool > right_bool,
    (Val::Number(left_number), Val::Number(right_number)) => left_number > right_number,
    (Val::String(left_string), Val::String(right_string)) => left_string > right_string,
    (Val::BigInt(left_bigint), Val::BigInt(right_bigint)) => left_bigint > right_bigint,
    _ => return format_err!("TODO"),
  }))
}

pub fn op_greater_eq(left: Val, right: Val) -> Result<Val, Val> {
  Ok(Val::Bool(match (left, right) {
    (Val::Undefined, Val::Undefined) => false,
    (Val::Null, Val::Null) => true,
    (Val::Bool(left_bool), Val::Bool(right_bool)) => left_bool >= right_bool,
    (Val::Number(left_number), Val::Number(right_number)) => left_number >= right_number,
    (Val::String(left_string), Val::String(right_string)) => left_string >= right_string,
    (Val::BigInt(left_bigint), Val::BigInt(right_bigint)) => left_bigint >= right_bigint,
    _ => return format_err!("TODO"),
  }))
}

pub fn op_nullish_coalesce(left: Val, right: Val) -> Result<Val, Val> {
  Ok(if left.is_nullish() { right } else { left })
}

pub fn op_optional_chain(left: Val, right: Val) -> Result<Val, Val> {
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

pub fn op_bit_and(left: Val, right: Val) -> Result<Val, Val> {
  match (left.as_bigint_data(), right.as_bigint_data()) {
    (Some(left_bigint), Some(right_bigint)) => Ok(Val::BigInt(left_bigint & right_bigint)),
    (Some(_), None) | (None, Some(_)) => {
      format_err!("TypeError: Cannot mix BigInt with other types")
    }
    _ => {
      let res_i32 = to_i32(left.to_number()) & to_i32(right.to_number());
      Ok(Val::Number(res_i32 as f64))
    }
  }
}

pub fn op_bit_or(left: Val, right: Val) -> Result<Val, Val> {
  match (left.as_bigint_data(), right.as_bigint_data()) {
    (Some(left_bigint), Some(right_bigint)) => Ok(Val::BigInt(left_bigint | right_bigint)),
    (Some(_), None) | (None, Some(_)) => {
      format_err!("TypeError: Cannot mix BigInt with other types")
    }
    _ => {
      let res_i32 = to_i32(left.to_number()) | to_i32(right.to_number());
      Ok(Val::Number(res_i32 as f64))
    }
  }
}

pub fn op_bit_not(input: Val) -> Val {
  match input.as_bigint_data() {
    Some(bigint) => Val::BigInt(!bigint),
    None => {
      let res_i32 = !to_i32(input.to_number());
      Val::Number(res_i32 as f64)
    }
  }
}

pub fn op_bit_xor(left: Val, right: Val) -> Result<Val, Val> {
  match (left.as_bigint_data(), right.as_bigint_data()) {
    (Some(left_bigint), Some(right_bigint)) => Ok(Val::BigInt(left_bigint ^ right_bigint)),
    (Some(_), None) | (None, Some(_)) => {
      format_err!("TypeError: Cannot mix BigInt with other types")
    }
    _ => {
      let res_i32 = to_i32(left.to_number()) ^ to_i32(right.to_number());
      Ok(Val::Number(res_i32 as f64))
    }
  }
}

pub fn op_left_shift(left: Val, right: Val) -> Result<Val, Val> {
  match (left.as_bigint_data(), right.as_bigint_data()) {
    (Some(left_bigint), Some(right_bigint)) => Ok(Val::BigInt(
      left_bigint << right_bigint.to_i64().expect("TODO"),
    )),
    (Some(_), None) | (None, Some(_)) => {
      format_err!("TypeError: Cannot mix BigInt with other types")
    }
    _ => {
      let res_i32 = to_i32(left.to_number()) << (to_u32(right.to_number()) & 0x1f);
      Ok(Val::Number(res_i32 as f64))
    }
  }
}

pub fn op_right_shift(left: Val, right: Val) -> Result<Val, Val> {
  match (left.as_bigint_data(), right.as_bigint_data()) {
    (Some(left_bigint), Some(right_bigint)) => {
      let right_i64 = right_bigint.to_i64().ok_or(format_string!("TODO"))?;
      Ok(Val::BigInt(left_bigint >> right_i64))
    }
    (Some(_), None) | (None, Some(_)) => {
      format_err!("TypeError: Cannot mix BigInt with other types")
    }
    _ => {
      let res_i32 = to_i32(left.to_number()) >> (to_u32(right.to_number()) & 0x1f);
      Ok(Val::Number(res_i32 as f64))
    }
  }
}

pub fn op_right_shift_unsigned(left: Val, right: Val) -> Result<Val, Val> {
  match (left.as_bigint_data(), right.as_bigint_data()) {
    (Some(_), Some(_)) => {
      format_err!("TypeError: BigInts don't support unsigned right shift")
    }
    (Some(_), None) | (None, Some(_)) => {
      format_err!("TypeError: Cannot mix BigInt with other types")
    }
    _ => {
      let res_u32 = to_u32(left.to_number()) >> (to_u32(right.to_number()) & 0x1f);
      Ok(Val::Number(res_u32 as f64))
    }
  }
}

pub fn op_typeof(input: Val) -> Val {
  use VsType::*;

  return Val::String(Rc::new(match input.typeof_() {
    Undefined => "undefined".to_string(),
    Null => "object".to_string(),
    Bool => "boolean".to_string(),
    Number => "number".to_string(),
    BigInt => "bigint".to_string(),
    String => "string".to_string(),
    Array => "object".to_string(),
    Object => "object".to_string(),
    Function => "function".to_string(),
    Class => "function".to_string(),
  }));
}

pub fn op_instance_of(_left: Val, _right: Val) -> Result<Val, Val> {
  std::panic!("Not implemented: op_instance_of");
}

pub fn op_in(_left: Val, _right: Val) -> Result<Val, Val> {
  std::panic!("Not implemented: op_in");
}

pub fn op_sub(left: Val, right: Val) -> Result<Val, Val> {
  return match left {
    Val::Void => format_err!("Internal: Shouldn't happen"), // TODO: Internal errors
    Val::Undefined => format_err!("TypeError: Cannot subscript undefined"),
    Val::Null => format_err!("TypeError: Cannot subscript null"),
    Val::Bool(_) => Ok(match right.val_to_string().as_str() {
      "toString" => Val::Static(&BOOL_TO_STRING),
      "valueOf" => Val::Static(&BOOL_VALUE_OF),
      _ => Val::Undefined,
    }),
    Val::Number(number) => Ok(op_sub_number(number, &right)),
    Val::BigInt(bigint) => Ok(op_sub_bigint(&bigint, &right)),
    Val::String(string_data) => Ok(op_sub_string(&string_data, &right)),
    Val::Array(array_data) => {
      let right_index = match right.to_index() {
        None => {
          // FIXME: Inefficient val_to_string() that gets duplicated
          // when subscripting the object
          if right.val_to_string() == "length" {
            return Ok(Val::Number(array_data.elements.len() as f64));
          }

          return Ok(array_data.object.sub(right));
        }
        Some(i) => i,
      };

      if right_index >= array_data.elements.len() {
        return Ok(Val::Undefined);
      }

      let res = array_data.elements[right_index].clone();

      return Ok(match res {
        Val::Void => Val::Undefined,
        _ => res,
      });
    }
    Val::Object(object_data) => Ok(object_data.sub(right)),
    Val::Function(_) | Val::Class(_) => Ok(Val::Undefined),
    Val::Static(s) => Ok(s.sub(right)),
    Val::Custom(custom_data) => Ok(custom_data.sub(right)),
  };
}

pub fn op_submov(target: &mut Val, subscript: Val, value: Val) -> Result<(), Val> {
  match target {
    Val::Void => format_err!("Internal: Shouldn't happen"), // TODO: Internal errors
    Val::Undefined => format_err!("Cannot assign to subscript of undefined"),
    Val::Null => format_err!("Cannot assign to subscript of null"),
    Val::Bool(_) => format_err!("Cannot assign to subscript of bool"),
    Val::Number(_) => format_err!("Cannot assign to subscript of number"),
    Val::BigInt(_) => format_err!("Cannot assign to subscript of bigint"),
    Val::String(_) => format_err!("Cannot assign to subscript of string"),
    Val::Array(array_data) => {
      let subscript_index = match subscript.to_index() {
        None => return format_err!("TODO: non-uint array subscript assignment"),
        Some(i) => i,
      };

      let array_data_mut = Rc::make_mut(array_data);

      if subscript_index < array_data_mut.elements.len() {
        array_data_mut.elements[subscript_index] = value;
      } else {
        if subscript_index - array_data_mut.elements.len() > 100 {
          return format_err!("TODO: Sparse arrays");
        }

        while subscript_index > array_data_mut.elements.len() {
          array_data_mut.elements.push(Val::Void);
        }

        array_data_mut.elements.push(value);
      }

      Ok(())
    }
    Val::Object(object_data) => {
      Rc::make_mut(object_data)
        .string_map
        .insert(subscript.val_to_string(), value);

      Ok(())
    }
    Val::Function(_) => format_err!("TODO: function subscript assignment"),
    Val::Class(_) => format_err!("Cannot assign to subscript of class"),
    Val::Static(_) => format_err!("Cannot assign to subscript of static value"),
    Val::Custom(_) => format_err!("TODO: Assign to subscript of custom value"),
  }
}

static BOOL_TO_STRING: NativeFunction = NativeFunction {
  fn_: |this: &mut Val, _args: Vec<Val>| -> Val {
    match &this {
      Val::Bool(b) => Val::String(Rc::new(b.to_string())),
      _ => std::panic!("TODO: Exceptions/bool indirection"),
    }
  },
};

static BOOL_VALUE_OF: NativeFunction = NativeFunction {
  fn_: |this: &mut Val, _args: Vec<Val>| -> Val {
    match &this {
      Val::Bool(b) => Val::Bool(*b),
      _ => std::panic!("TODO: Exceptions/bool indirection"),
    }
  },
};
