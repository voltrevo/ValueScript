use num_bigint::BigInt;

use crate::{
  builtins::internal_error_builtin::ToInternalError,
  native_function::{native_fn, NativeFunction},
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

static TO_STRING: NativeFunction = native_fn(|this, params| {
  Ok(match this.get() {
    Val::BigInt(bigint) => match params.first() {
      Some(_) => {
        return Err("TODO: toString with radix".to_internal_error());
      }

      None => bigint.clone().to_val().to_string().to_val(),
    },
    _ => return Err("TODO: bigint indirection".to_internal_error()),
  })
});

static VALUE_OF: NativeFunction = native_fn(|this, _params| {
  Ok(match this.get() {
    Val::BigInt(bigint) => Val::BigInt(bigint.clone()),
    _ => return Err("TODO: bigint indirection".to_internal_error()),
  })
});
