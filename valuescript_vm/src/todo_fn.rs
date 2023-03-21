use std::rc::Rc;

use crate::{format_err, native_function::NativeFunction, vs_value::Val};

pub static TODO: NativeFunction = NativeFunction {
  fn_: |this: &mut Val, _params: Vec<Val>| -> Result<Val, Val> {
    match this {
      Val::Number(_number) => return format_err!("TODO: locale"),
      _ => return format_err!("number indirection"),
    }
  },
};
