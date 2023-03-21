use std::rc::Rc;

use num_bigint::BigInt;

use crate::{
  format_err,
  native_function::NativeFunction,
  todo_fn::TODO,
  vs_value::{Val, ValTrait},
};

pub fn op_sub_bigint(_bigint: &BigInt, subscript: &Val) -> Val {
  match subscript.val_to_string().as_str() {
    "toLocaleString" => Val::Static(&TODO),
    "toString" => Val::Static(&TO_STRING),
    "valueOf" => Val::Static(&VALUE_OF),
    _ => Val::Undefined,
  }
}

static TO_STRING: NativeFunction = NativeFunction {
  fn_: |this: &mut Val, params: Vec<Val>| -> Result<Val, Val> {
    Ok(match this {
      Val::BigInt(_) => match params.get(0) {
        Some(_) => {
          return format_err!("TODO: toString with radix");
        }

        None => Val::String(Rc::new(this.val_to_string())),
      },
      _ => return format_err!("TODO: bigint indirection"),
    })
  },
};

static VALUE_OF: NativeFunction = NativeFunction {
  fn_: |this: &mut Val, _params: Vec<Val>| -> Result<Val, Val> {
    Ok(match this {
      Val::BigInt(bigint) => Val::BigInt(bigint.clone()),
      _ => return format_err!("TODO: bigint indirection"),
    })
  },
};
