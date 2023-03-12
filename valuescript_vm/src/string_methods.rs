use std::rc::Rc;

use crate::{helpers::to_wrapping_index, native_function::NativeFunction, vs_value::Val, ValTrait};

pub fn get_string_method(method: &str) -> Val {
  match method {
    "at" => Val::Static(&AT),
    "charAt" => Val::Static(&CHAR_AT),
    "charCodeAt" => Val::Static(&CHAR_CODE_AT),
    _ => Val::Undefined,
  }
}

static AT: NativeFunction = NativeFunction {
  fn_: |this: &mut Val, params: Vec<Val>| -> Val {
    match this {
      Val::String(string_data) => {
        let string_bytes = string_data.as_bytes();

        let index = match to_wrapping_index(params.get(0), string_bytes.len()) {
          None => return Val::Undefined,
          Some(i) => i,
        };

        let byte = string_bytes[index];

        Val::String(Rc::new(String::from_utf8_lossy(&[byte]).into_owned()))
      }
      _ => std::panic!("Not implemented: exceptions/string indirection"),
    }
  },
};

static CHAR_AT: NativeFunction = NativeFunction {
  fn_: |this: &mut Val, params: Vec<Val>| -> Val {
    match this {
      Val::String(string_data) => match byte_at(string_data, params.get(0)) {
        Some(byte) => string_from_byte(byte),
        None => Val::String(Rc::new(String::from(""))),
      },
      _ => std::panic!("Not implemented: exceptions/string indirection"),
    }
  },
};

static CHAR_CODE_AT: NativeFunction = NativeFunction {
  fn_: |this: &mut Val, params: Vec<Val>| -> Val {
    match this {
      Val::String(string_data) => match byte_at(string_data, params.get(0)) {
        Some(byte) => Val::Number(byte as f64),
        None => Val::Number(std::f64::NAN),
      },
      _ => std::panic!("Not implemented: exceptions/string indirection"),
    }
  },
};

fn byte_at(string: &String, index_param: Option<&Val>) -> Option<u8> {
  let mut index = match index_param {
    Some(i) => i.to_number(),
    _ => 0 as f64,
  };

  if index.is_nan() {
    index = 0 as f64;
  }

  index = index.trunc();

  if index < 0_f64 {
    return None;
  }

  let index_usize = index as usize;

  let string_bytes = string.as_bytes();

  if index_usize >= string_bytes.len() {
    return None;
  }

  Some(string_bytes[index_usize])
}

fn string_from_byte(byte: u8) -> Val {
  Val::String(Rc::new(String::from_utf8_lossy(&[byte]).into_owned()))
}
