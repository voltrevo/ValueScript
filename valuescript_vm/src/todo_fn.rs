use crate::{native_function::NativeFunction, vs_value::Val};

pub static TODO: NativeFunction = NativeFunction {
  fn_: |this: &mut Val, _params: Vec<Val>| -> Result<Val, Val> {
    match this {
      Val::Number(_number) => panic!("TODO: locale"),
      _ => panic!("TODO: exceptions/number indirection"),
    }
  },
};
