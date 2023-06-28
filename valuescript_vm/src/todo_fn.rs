use crate::{
  builtins::internal_error_builtin::ToInternalError,
  native_function::{native_fn, NativeFunction},
};

pub static TODO: NativeFunction = native_fn(|_, _| Err("TODO".to_internal_error()));
