use std::rc::Rc;

use crate::{helpers::to_wrapping_index, native_function::NativeFunction, vs_value::Val, ValTrait};

pub fn op_sub_string(string_data: &Rc<String>, subscript: &Val) -> Val {
  let right_index = match subscript.to_index() {
    None => {
      let method = subscript.val_to_string();
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

  string_from_byte(string_bytes[right_index])
}

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
  // TODO: Val::Strings need to change to not use rust's string type,
  // because they need to represent an actual byte array underneath. This
  // occurs for invalid utf8 sequences which are getting converted to U+FFFD
  // here. To be analogous to js, the information of the actual byte needs
  // to be preserved, but that can't be represented in rust's string type.
  Val::String(Rc::new(String::from_utf8_lossy(&[byte]).into_owned()))
}
