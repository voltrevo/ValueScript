use std::rc::Rc;

use crate::{helpers::to_wrapping_index, native_function::NativeFunction, vs_value::Val, ValTrait};

pub fn op_sub_string(string_data: &Rc<String>, subscript: &Val) -> Val {
  let right_index = match subscript.to_index() {
    None => {
      let method = subscript.val_to_string();
      let method_str = method.as_str();

      return match method_str {
        "length" => Val::Number(string_data.as_bytes().len() as f64),
        _ => get_string_method(method_str),
      };
    }
    Some(i) => i,
  };

  let string_bytes = string_data.as_bytes();

  if right_index >= string_bytes.len() {
    return Val::Undefined;
  }

  match unicode_at(string_bytes, right_index) {
    Some(string) => Val::String(Rc::new(string)),
    None => Val::String(Rc::new(String::from(""))),
  }
}

pub fn get_string_method(method: &str) -> Val {
  // Not supported: charAt, charCodeAt.
  //
  // These methods are inherently about utf16, which is not how strings work in ValueScript. They
  // also have some particularly strange behavior, like:
  //
  //     "foo".charAt(NaN) // "f"
  //
  // Usually we include JavaScript behavior as much as possible, but since ValueScript strings are
  // utf8, there's more license to reinterpret strings and leave out things like this which aren't
  // desirable.

  match method {
    "at" => Val::Static(&AT),
    // "charAt" => Val::Static(&CHAR_AT),
    // "charCodeAt" => Val::Static(&CHAR_CODE_AT),
    "codePointAt" => Val::Static(&CODE_POINT_AT),
    "concat" => Val::Static(&CONCAT),
    "endsWith" => Val::Static(&ENDS_WITH),
    "includes" => Val::Static(&INCLUDES),
    "indexOf" => Val::Static(&INDEX_OF),
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

        match unicode_at(string_bytes, index) {
          Some(string) => Val::String(Rc::new(string)),
          None => Val::String(Rc::new(String::from(""))),
        }
      }
      _ => std::panic!("Not implemented: exceptions/string indirection"),
    }
  },
};

static CODE_POINT_AT: NativeFunction = NativeFunction {
  fn_: |this: &mut Val, params: Vec<Val>| -> Val {
    match this {
      Val::String(string_data) => {
        let string_bytes = string_data.as_bytes();

        let index = match params.get(0) {
          Some(i) => match i.to_index() {
            None => return Val::Undefined,
            Some(i) => i,
          },
          _ => return Val::Undefined,
        };

        match code_point_at(string_bytes, index) {
          Some(code_point) => Val::Number(code_point as f64),
          None => Val::Undefined,
        }
      }
      _ => std::panic!("Not implemented: exceptions/string indirection"),
    }
  },
};

static CONCAT: NativeFunction = NativeFunction {
  fn_: |this: &mut Val, params: Vec<Val>| -> Val {
    match this {
      Val::String(string_data) => {
        let mut result = string_data.as_str().to_string();

        for param in params {
          result.push_str(param.val_to_string().as_str());
        }

        Val::String(Rc::new(result))
      }
      _ => std::panic!("Not implemented: exceptions/string indirection"),
    }
  },
};

static ENDS_WITH: NativeFunction = NativeFunction {
  fn_: |this: &mut Val, params: Vec<Val>| -> Val {
    match this {
      Val::String(string_data) => {
        let string_bytes = string_data.as_bytes();

        let search_string = match params.get(0) {
          Some(s) => s.val_to_string(),
          _ => return Val::Bool(false),
        };

        let end_pos = match params.get(1) {
          Some(p) => match p.to_index() {
            // FIXME: Using to_index for end_pos is not quite right (eg -1 should be 0)
            None => return Val::Bool(false),
            Some(i) => std::cmp::min(i, string_bytes.len()),
          },
          _ => string_bytes.len(),
        };

        let search_bytes = search_string.as_bytes();

        let search_length = search_bytes.len();

        if search_length > end_pos {
          return Val::Bool(false);
        }

        let start_index = end_pos - search_length;

        for i in 0..search_length {
          if string_bytes[start_index + i] != search_bytes[i] {
            return Val::Bool(false);
          }
        }

        Val::Bool(true)
      }
      _ => std::panic!("Not implemented: exceptions/string indirection"),
    }
  },
};

static INCLUDES: NativeFunction = NativeFunction {
  fn_: |this: &mut Val, params: Vec<Val>| -> Val {
    match this {
      Val::String(string_data) => {
        let string_bytes = string_data.as_bytes();

        let search_string = match params.get(0) {
          Some(s) => s.val_to_string(),
          _ => return Val::Bool(false),
        };

        let search_bytes = search_string.as_bytes();

        let start_pos = match params.get(1) {
          Some(p) => match p.to_index() {
            // FIXME: to_index isn't quite right here
            Some(i) => i,
            None => return Val::Bool(false),
          },
          _ => 0,
        };

        match index_of(string_bytes, search_bytes, start_pos) {
          Some(_) => Val::Bool(true),
          None => Val::Bool(false),
        }
      }
      _ => std::panic!("Not implemented: exceptions/string indirection"),
    }
  },
};

static INDEX_OF: NativeFunction = NativeFunction {
  fn_: |this: &mut Val, params: Vec<Val>| -> Val {
    match this {
      Val::String(string_data) => {
        let string_bytes = string_data.as_bytes();

        let search_string = match params.get(0) {
          Some(s) => s.val_to_string(),
          _ => return Val::Number(-1.0),
        };

        let search_bytes = search_string.as_bytes();

        let start_pos = match params.get(1) {
          Some(p) => match p.to_index() {
            // FIXME: to_index isn't quite right here
            Some(i) => i,
            None => return Val::Number(-1.0),
          },
          _ => 0,
        };

        match index_of(string_bytes, search_bytes, start_pos) {
          Some(i) => Val::Number(i as f64),
          None => Val::Number(-1.0),
        }
      }
      _ => std::panic!("Not implemented: exceptions/string indirection"),
    }
  },
};

fn index_of(string_bytes: &[u8], search_bytes: &[u8], start_pos: usize) -> Option<usize> {
  let search_length = search_bytes.len();

  if start_pos + search_length > string_bytes.len() {
    // TODO: If search_length is 0, we should apparently return the length of the string instead of
    // returning none (why??)
    return None;
  }

  if search_length == 0 {
    return Some(0);
  }

  for i in start_pos..(string_bytes.len() - search_length + 1) {
    let mut found = true;

    for j in 0..search_length {
      if string_bytes[i + j] != search_bytes[j] {
        found = false;
        break;
      }
    }

    if found {
      return Some(i);
    }
  }

  None
}

fn unicode_at(bytes: &[u8], index: usize) -> Option<String> {
  match code_point_at(bytes, index) {
    Some(code_point) => Some(
      std::char::from_u32(code_point)
        .expect("Invalid code point") // TODO: Find out if this is reachable and what to do about it
        .to_string(),
    ),
    None => None,
  }
}

fn code_point_at(bytes: &[u8], index: usize) -> Option<u32> {
  if index >= bytes.len() {
    return None;
  }

  let byte = bytes[index];

  let leading_ones = byte.leading_ones() as usize;

  if leading_ones == 0 {
    return Some(byte as u32);
  }

  if leading_ones == 1 || leading_ones > 4 || index + leading_ones > bytes.len() {
    return None;
  }

  let mut value = (byte & (0x7F >> leading_ones)) as u32;

  for i in 1..leading_ones {
    let next_byte = bytes[index + i];

    if next_byte.leading_ones() != 1 {
      return None;
    }

    value = (value << 6) | (next_byte & 0x3F) as u32;
  }

  Some(value)
}
