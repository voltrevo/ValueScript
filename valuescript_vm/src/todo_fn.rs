use crate::{
  builtins::error_builtin::ToError,
  native_function::{NativeFunction, ThisWrapper},
  vs_value::Val,
};

pub static TODO: NativeFunction = NativeFunction {
  fn_: |_: ThisWrapper, _params: Vec<Val>| -> Result<Val, Val> { Err("TODO".to_error()) },
};
