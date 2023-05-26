use num_bigint::BigInt;

use crate::{
  builtins::error_builtin::ToError,
  native_function::{NativeFunction, ThisWrapper},
  todo_fn::TODO,
  vs_value::{ToValString, Val, ValTrait},
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
  fn_: |this: ThisWrapper, params: Vec<Val>| -> Result<Val, Val> {
    Ok(match this.get() {
      Val::BigInt(_) => match params.get(0) {
        Some(_) => {
          return Err("TODO: toString with radix".to_error());
        }

        None => this.get().to_val_string(),
      },
      _ => return Err("TODO: bigint indirection".to_error()),
    })
  },
};

static VALUE_OF: NativeFunction = NativeFunction {
  fn_: |this: ThisWrapper, _params: Vec<Val>| -> Result<Val, Val> {
    Ok(match this.get() {
      Val::BigInt(bigint) => Val::BigInt(bigint.clone()),
      _ => return Err("TODO: bigint indirection".to_error()),
    })
  },
};
