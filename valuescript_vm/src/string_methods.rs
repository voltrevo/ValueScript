use std::rc::Rc;

use crate::{helpers::to_wrapping_index, native_function::NativeFunction, vs_value::Val};

pub fn get_string_method(method: &str) -> Val {
  match method {
    "at" => Val::Static(&AT),
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
