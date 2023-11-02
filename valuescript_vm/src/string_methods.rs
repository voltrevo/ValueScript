use std::{rc::Rc, str::Chars};

use crate::{
  builtins::internal_error_builtin::ToInternalError,
  helpers::{to_wrapping_index, to_wrapping_index_clamped},
  iteration::string_iterator::StringIterator,
  native_function::{native_fn, NativeFunction},
  vs_symbol::VsSymbol,
  vs_value::{ToDynamicVal, ToVal, Val},
  ValTrait,
};

pub fn op_sub_string(string_data: &Rc<str>, subscript: &Val) -> Val {
  if let Some(subscript) = subscript.to_index() {
    let string_bytes = string_data.as_bytes();

    if subscript >= string_bytes.len() {
      return Val::Undefined;
    }

    return match unicode_at(string_bytes, string_bytes.len(), subscript) {
      Some(char) => char.to_string().to_val(),
      None => "".to_val(),
    };
  }

  if let Val::Symbol(subscript) = subscript {
    match subscript {
      VsSymbol::ITERATOR => return VALUES.to_val(),
    }
  }

  let method = subscript.to_string();
  let method_str = method.as_str();

  return match method_str {
    "length" => Val::Number(string_data.as_bytes().len() as f64),
    _ => get_string_method(method_str),
  };
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
    "at" => &AT,
    // "charAt" => &CHAR_AT,
    // "charCodeAt" => &CHAR_CODE_AT,
    "codePointAt" => &CODE_POINT_AT,
    "concat" => &CONCAT,
    "endsWith" => &ENDS_WITH,
    "includes" => &INCLUDES,
    "indexOf" => &INDEX_OF,
    "lastIndexOf" => &LAST_INDEX_OF,
    "localeCompare" => &TODO_LOCALE, // (TODO)
    "match" => &TODO_REGEXES,        // (TODO: regex)
    "matchAll" => &TODO_REGEXES,     // (TODO: regex)
    "normalize" => &NORMALIZE,       // (TODO)
    "padEnd" => &PAD_END,
    "padStart" => &PAD_START,
    "repeat" => &REPEAT,
    "replace" => &TODO_REGEXES,    // (TODO: regex)
    "replaceAll" => &TODO_REGEXES, // (TODO: regex)
    "search" => &TODO_REGEXES,     // (TODO: regex)
    "slice" => &SLICE,
    "split" => &SPLIT,
    "startsWith" => &STARTS_WITH,
    "substring" => &SUBSTRING,
    "toLocaleLowerCase" => &TODO_LOCALE,
    "toLocaleUpperCase" => &TODO_LOCALE,
    "toLowerCase" => &TO_LOWER_CASE,
    "toString" => &TO_STRING,
    "toUpperCase" => &TO_UPPER_CASE,
    "trim" => &TRIM,
    "trimEnd" => &TRIM_END,
    "trimStart" => &TRIM_START,
    "valueOf" => &VALUE_OF,
    _ => return Val::Undefined,
  }
  .to_val()
}

static AT: NativeFunction = native_fn(|this, params| {
  Ok(match this.get() {
    Val::String(string_data) => {
      let string_bytes = string_data.as_bytes();

      let index = match to_wrapping_index(params.get(0), string_bytes.len()) {
        None => return Ok(Val::Undefined),
        Some(i) => i,
      };

      match unicode_at(string_bytes, string_bytes.len(), index) {
        Some(char) => char.to_string().to_val(),
        None => "".to_val(),
      }
    }
    _ => return Err("string indirection".to_internal_error()),
  })
});

static CODE_POINT_AT: NativeFunction = native_fn(|this, params| {
  Ok(match this.get() {
    Val::String(string_data) => {
      let string_bytes = string_data.as_bytes();

      let index = match params.get(0) {
        Some(i) => match i.to_index() {
          None => return Ok(Val::Undefined),
          Some(i) => i,
        },
        _ => return Ok(Val::Undefined),
      };

      match code_point_at(string_bytes, string_bytes.len(), index) {
        Some(code_point) => Val::Number(code_point as f64),
        None => Val::Undefined,
      }
    }
    _ => return Err("string indirection".to_internal_error()),
  })
});

static CONCAT: NativeFunction = native_fn(|this, params| {
  Ok(match this.get() {
    Val::String(string_data) => {
      let mut result = string_data.to_string();

      for param in params {
        match param {
          Val::String(str) => result.push_str(&str),
          _ => result.push_str(param.to_string().as_str()),
        };
      }

      result.to_val()
    }
    _ => return Err("string indirection".to_internal_error()),
  })
});

static ENDS_WITH: NativeFunction = native_fn(|this, params| {
  Ok(match this.get() {
    Val::String(string_data) => {
      let string_bytes = string_data.as_bytes();

      let search_string = match params.get(0) {
        Some(s) => s.to_string(),
        _ => return Ok(Val::Bool(false)),
      };

      let end_pos = match params.get(1) {
        Some(p) => match p.to_index() {
          // FIXME: Using to_index for end_pos is not quite right (eg -1 should be 0)
          None => return Ok(Val::Bool(false)),
          Some(i) => std::cmp::min(i, string_bytes.len()),
        },
        _ => string_bytes.len(),
      };

      let search_bytes = search_string.as_bytes();

      let search_length = search_bytes.len();

      if search_length > end_pos {
        return Ok(Val::Bool(false));
      }

      let start_index = end_pos - search_length;

      for i in 0..search_length {
        if string_bytes[start_index + i] != search_bytes[i] {
          return Ok(Val::Bool(false));
        }
      }

      Val::Bool(true)
    }
    _ => return Err("string indirection".to_internal_error()),
  })
});

static INCLUDES: NativeFunction = native_fn(|this, params| {
  Ok(match this.get() {
    Val::String(string_data) => {
      let string_bytes = string_data.as_bytes();

      let search_string = match params.get(0) {
        Some(s) => s.to_string(),
        _ => return Ok(Val::Bool(false)),
      };

      let search_bytes = search_string.as_bytes();

      let start_pos = match params.get(1) {
        Some(p) => match p.to_index() {
          // FIXME: to_index isn't quite right here
          Some(i) => i,
          None => return Ok(Val::Bool(false)),
        },
        _ => 0,
      };

      match index_of(string_bytes, search_bytes, start_pos) {
        Some(_) => Val::Bool(true),
        None => Val::Bool(false),
      }
    }
    _ => return Err("string indirection".to_internal_error()),
  })
});

static INDEX_OF: NativeFunction = native_fn(|this, params| {
  Ok(match this.get() {
    Val::String(string_data) => {
      let string_bytes = string_data.as_bytes();

      let search_string = match params.get(0) {
        Some(s) => s.to_string(),
        _ => return Ok(Val::Number(-1.0)),
      };

      let search_bytes = search_string.as_bytes();

      let start_pos = match params.get(1) {
        Some(p) => match p.to_index() {
          // FIXME: to_index isn't quite right here
          Some(i) => i,
          None => return Ok(Val::Number(-1.0)),
        },
        _ => 0,
      };

      match index_of(string_bytes, search_bytes, start_pos) {
        Some(i) => Val::Number(i as f64),
        None => Val::Number(-1.0),
      }
    }
    _ => return Err("string indirection".to_internal_error()),
  })
});

static LAST_INDEX_OF: NativeFunction = native_fn(|this, params| {
  Ok(match this.get() {
    Val::String(string_data) => {
      let string_bytes = string_data.as_bytes();

      let search_string = match params.get(0) {
        Some(s) => s.to_string(),
        _ => return Ok(Val::Number(-1.0)),
      };

      let search_bytes = search_string.as_bytes();

      let at_least_pos = match params.get(1) {
        Some(p) => match p.to_index() {
          // FIXME: to_index isn't quite right here
          Some(i) => i,
          None => return Ok(Val::Number(-1.0)),
        },
        _ => 0,
      };

      match last_index_of(string_bytes, search_bytes, at_least_pos) {
        Some(i) => Val::Number(i as f64),
        None => Val::Number(-1.0),
      }
    }
    _ => return Err("string indirection".to_internal_error()),
  })
});

static TODO_LOCALE: NativeFunction = native_fn(|this, _params| {
  // TODO: Ok(...)
  match this.get() {
    Val::String(_string_data) => Err("TODO: locale".to_internal_error()),
    _ => Err("string indirection".to_internal_error()),
  }
});

static TODO_REGEXES: NativeFunction = native_fn(|this, _params| {
  // TODO: Ok(...)
  match this.get() {
    Val::String(_string_data) => Err("TODO: regexes".to_internal_error()),
    _ => Err("string indirection".to_internal_error()),
  }
});

static NORMALIZE: NativeFunction = native_fn(|this, _params| {
  // TODO: Ok(...)
  match this.get() {
    Val::String(_string_data) => {
      // Consider https://docs.rs/unicode-normalization/latest/unicode_normalization/
      Err("TODO: normalize".to_internal_error())
    }
    _ => Err("string indirection".to_internal_error()),
  }
});

// TODO: JS has some locale-specific behavior, not sure yet how we should deal with that
static PAD_END: NativeFunction = native_fn(|this, params| {
  Ok(match this.get() {
    Val::String(string_data) => {
      let target_length = match params.get(0) {
        Some(p) => match p.to_index() {
          Some(i) => i,
          None => return Ok(Val::String(string_data.clone())),
        },
        _ => return Ok(Val::String(string_data.clone())),
      };

      if target_length <= string_data.as_bytes().len() {
        return Ok(Val::String(string_data.clone()));
      }

      let mut string = string_data.to_string();

      let pad_string = match params.get(1) {
        Some(s) => s.to_string(),
        _ => " ".to_string(),
      };

      let mut length_deficit = target_length - string.as_bytes().len();

      let whole_copies = length_deficit / pad_string.as_bytes().len();

      for _ in 0..whole_copies {
        string.push_str(&pad_string);
      }

      length_deficit -= whole_copies * pad_string.as_bytes().len();

      if length_deficit > 0 {
        for c in pad_string.chars() {
          let c_len = c.len_utf8();

          if c_len > length_deficit {
            break;
          }

          string.push(c);
          length_deficit -= c_len;
        }
      }

      string.to_val()
    }
    _ => return Err("string indirection".to_internal_error()),
  })
});

// TODO: JS has some locale-specific behavior, not sure yet how we should deal with that
static PAD_START: NativeFunction = native_fn(|this, params| {
  Ok(match this.get() {
    Val::String(string_data) => {
      let target_length = match params.get(0) {
        Some(p) => match p.to_index() {
          Some(i) => i,
          None => return Ok(Val::String(string_data.clone())),
        },
        _ => return Ok(Val::String(string_data.clone())),
      };

      if target_length <= string_data.as_bytes().len() {
        return Ok(Val::String(string_data.clone()));
      }

      let pad_string = match params.get(1) {
        Some(s) => s.to_string(),
        _ => " ".to_string(),
      };

      let mut length_deficit = target_length - string_data.as_bytes().len();

      let whole_copies = length_deficit / pad_string.as_bytes().len();

      let mut prefix = String::new();

      for _ in 0..whole_copies {
        prefix.push_str(&pad_string);
      }

      length_deficit -= whole_copies * pad_string.as_bytes().len();

      if length_deficit > 0 {
        for c in pad_string.chars() {
          let c_len = c.len_utf8();

          if c_len > length_deficit {
            break;
          }

          prefix.push(c);
          length_deficit -= c_len;
        }
      }

      prefix.push_str(&string_data);

      prefix.to_val()
    }
    _ => return Err("string indirection".to_internal_error()),
  })
});

static REPEAT: NativeFunction = native_fn(|this, params| {
  Ok(match this.get() {
    Val::String(string_data) => {
      let count = match params.get(0) {
        Some(p) => match p.to_index() {
          Some(i) => i,
          None => return Ok(Val::String(string_data.clone())),
        },
        _ => return Ok(Val::String(string_data.clone())),
      };

      let mut result = String::new();

      for _ in 0..count {
        result.push_str(&string_data);
      }

      result.to_val()
    }
    _ => return Err("string indirection".to_internal_error()),
  })
});

static SLICE: NativeFunction = native_fn(|this, params| {
  Ok(match this.get() {
    Val::String(string_data) => {
      let string_bytes = string_data.as_bytes();

      let start = match params.get(0) {
        None => 0,
        Some(v) => to_wrapping_index_clamped(v, string_bytes.len()),
      };

      let end = match params.get(1) {
        None => string_bytes.len() as isize,
        Some(v) => to_wrapping_index_clamped(v, string_bytes.len()),
      };

      let mut new_string = String::new();

      // FIXME: This is a slow way of doing it. Part of the reason is that we're using rust's
      // string type, so we can't just find the relevant byte range and copy it in one go.
      for i in start..end {
        if let Some(c) = unicode_at(string_bytes, end as usize, i as usize) {
          new_string.push(c)
        }
      }

      new_string.to_val()
    }
    _ => return Err("string indirection".to_internal_error()),
  })
});

static SPLIT: NativeFunction = native_fn(|this, params| {
  Ok(match this.get() {
    Val::String(string_data) => {
      let separator = match params.get(0) {
        Some(s) => s.to_string(), // TODO: Regexes
        None => return Ok(Val::String(string_data.clone())),
      };

      let limit = match params.get(1) {
        // FIXME: to_index isn't quite right
        Some(l) => match l.to_index() {
          Some(i) => i,
          None => string_data.as_bytes().len() + 1,
        },
        None => string_data.as_bytes().len() + 1,
      };

      let mut result = Vec::<Val>::new();

      if limit == 0 {
        return Ok(result.to_val());
      }

      if separator.is_empty() {
        for c in string_data.chars() {
          result.push(c.to_val());

          if result.len() == limit {
            break;
          }
        }

        return Ok(result.to_val());
      }

      let mut part = String::new();
      let mut str_chars = string_data.chars();

      loop {
        if match_chars(&mut str_chars, &separator) {
          let mut new_part = String::new();
          std::mem::swap(&mut new_part, &mut part);
          result.push(new_part.to_val());

          if result.len() == limit {
            break;
          }
        } else {
          match str_chars.next() {
            Some(c) => part.push(c),
            None => {
              result.push(part.to_val());
              break;
            }
          }
        }
      }

      result.to_val()
    }
    _ => return Err("string indirection".to_internal_error()),
  })
});

static STARTS_WITH: NativeFunction = native_fn(|this, params| {
  Ok(match this.get() {
    Val::String(string_data) => {
      let string_bytes = string_data.as_bytes();

      let search_string = match params.get(0) {
        Some(s) => s.to_string(),
        _ => return Ok(Val::Bool(false)),
      };

      let pos = match params.get(1) {
        Some(p) => match p.to_index() {
          // FIXME: Using to_index is not quite right
          None => return Ok(Val::Bool(false)),
          Some(i) => std::cmp::min(i, string_bytes.len()),
        },
        _ => 0,
      };

      let search_bytes = search_string.as_bytes();

      let search_length = search_bytes.len();

      if search_length > string_bytes.len() - pos {
        return Ok(Val::Bool(false));
      }

      for i in 0..search_length {
        if string_bytes[pos + i] != search_bytes[i] {
          return Ok(Val::Bool(false));
        }
      }

      Val::Bool(true)
    }
    _ => return Err("string indirection".to_internal_error()),
  })
});

static SUBSTRING: NativeFunction = native_fn(|this, params| {
  Ok(match this.get() {
    Val::String(string_data) => {
      let string_bytes = string_data.as_bytes();

      let start = match params.get(0) {
        Some(v) => match v.to_index() {
          Some(i) => std::cmp::min(i, string_bytes.len()),
          None => 0,
        },
        None => 0,
      };

      let end = match params.get(1) {
        Some(v) => match v.to_index() {
          Some(i) => std::cmp::min(i, string_bytes.len()),
          None => string_bytes.len(),
        },
        None => string_bytes.len(),
      };

      let substring_start = std::cmp::min(start, end);
      let substring_end = std::cmp::max(start, end);

      let mut new_string = String::new();

      // FIXME: This is a slow way of doing it. Part of the reason is that we're using rust's
      // string type, so we can't just find the relevant byte range and copy it in one go.
      for i in substring_start..substring_end {
        if let Some(c) = unicode_at(string_bytes, substring_end, i) {
          new_string.push(c)
        }
      }

      new_string.to_val()
    }
    _ => return Err("string indirection".to_internal_error()),
  })
});

static TO_LOWER_CASE: NativeFunction = native_fn(|this, _params| {
  Ok(match this.get() {
    Val::String(string_data) => string_data.to_lowercase().to_val(),
    _ => return Err("string indirection".to_internal_error()),
  })
});

static TO_STRING: NativeFunction = native_fn(|this, _params| {
  Ok(match this.get() {
    Val::String(string_data) => Val::String(string_data.clone()),
    _ => return Err("string indirection".to_internal_error()),
  })
});

static TO_UPPER_CASE: NativeFunction = native_fn(|this, _params| {
  Ok(match this.get() {
    Val::String(string_data) => string_data.to_uppercase().to_val(),
    _ => return Err("string indirection".to_internal_error()),
  })
});

static TRIM: NativeFunction = native_fn(|this, _params| {
  Ok(match this.get() {
    Val::String(string_data) => string_data.trim().to_val(),
    _ => return Err("string indirection".to_internal_error()),
  })
});

static TRIM_END: NativeFunction = native_fn(|this, _params| {
  Ok(match this.get() {
    Val::String(string_data) => string_data.trim_end().to_val(),
    _ => return Err("string indirection".to_internal_error()),
  })
});

static TRIM_START: NativeFunction = native_fn(|this, _params| {
  Ok(match this.get() {
    Val::String(string_data) => string_data.trim_start().to_val(),
    _ => return Err("string indirection".to_internal_error()),
  })
});

static VALUE_OF: NativeFunction = native_fn(|this, _params| {
  Ok(match this.get() {
    Val::String(string_data) => Val::String(string_data.clone()),
    _ => return Err("string indirection".to_internal_error()),
  })
});

static VALUES: NativeFunction = native_fn(|this, _params| match this.get() {
  Val::String(string_data) => Ok(StringIterator::new(string_data.clone()).to_dynamic_val()),
  _ => Err("string indirection".to_internal_error()),
});

/**
 * Tries to match str_chars_param against matcher.
 * - Successful match: Advances str_chars_param and returns true.
 * - Unsuccessful match: Does not advance str_chars_param and returns false.
 */
fn match_chars(str_chars_param: &mut Chars, matcher: &str) -> bool {
  let mut str_chars = str_chars_param.clone();
  let mut matcher_chars = matcher.chars();

  loop {
    let matcher_char = match matcher_chars.next() {
      Some(c) => c,
      None => {
        *str_chars_param = str_chars;
        return true;
      }
    };

    let str_char = match str_chars.next() {
      Some(c) => c,
      None => return false,
    };

    if str_char != matcher_char {
      return false;
    }
  }
}

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

  'outer: for i in start_pos..=(string_bytes.len() - search_length) {
    for j in 0..search_length {
      if string_bytes[i + j] != search_bytes[j] {
        continue 'outer;
      }
    }

    return Some(i);
  }

  None
}

fn last_index_of(string_bytes: &[u8], search_bytes: &[u8], at_least_pos: usize) -> Option<usize> {
  let search_length = search_bytes.len();

  if search_length > string_bytes.len() {
    return None;
  }

  if search_length == 0 {
    return Some(string_bytes.len());
  }

  'outer: for i in (at_least_pos..=string_bytes.len() - search_length).rev() {
    for j in 0..search_length {
      if string_bytes[i + j] != search_bytes[j] {
        continue 'outer;
      }
    }

    return Some(i);
  }

  None
}

pub fn unicode_at(bytes: &[u8], len: usize, index: usize) -> Option<char> {
  code_point_at(bytes, len, index)
    .map(|code_point| std::char::from_u32(code_point).expect("Invalid code point"))
}

fn code_point_at(bytes: &[u8], len: usize, index: usize) -> Option<u32> {
  if index >= len {
    return None;
  }

  let byte = bytes[index];

  let leading_ones = byte.leading_ones() as usize;

  if leading_ones == 0 {
    return Some(byte as u32);
  }

  if leading_ones == 1 || leading_ones > 4 || index + leading_ones > len {
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
