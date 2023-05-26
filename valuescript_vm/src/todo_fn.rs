use crate::{
  builtins::error_builtin::ToError,
  native_function::{native_fn, NativeFunction},
};

pub static TODO: NativeFunction = native_fn(|_, _| Err("TODO".to_error()));
