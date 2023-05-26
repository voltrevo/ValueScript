use num_bigint::BigInt;

use crate::{
  builtins::error_builtin::ToError,
  native_function::{NativeFunction, ThisWrapper},
  todo_fn::TODO,
  vs_value::{ToVal, Val},
};

pub fn op_sub_bigint(_bigint: &BigInt, subscript: &Val) -> Val {
  match subscript.to_string().as_str() {
    "toLocaleString" => &TODO,
    "toString" => &TO_STRING,
    "valueOf" => &VALUE_OF,
    _ => return Val::Undefined,
  }
  .to_val()
}

static TO_STRING: NativeFunction = NativeFunction {
  fn_: |this: ThisWrapper, params: Vec<Val>| -> Result<Val, Val> {
    Ok(match this.get() {
      Val::BigInt(bigint) => match params.get(0) {
        Some(_) => {
          return Err("TODO: toString with radix".to_error());
        }

        None => bigint.clone().to_val().to_string().to_val(),
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
