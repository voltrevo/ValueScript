use crate::native_function::{native_fn, NativeFunction};

pub static RETURN_THIS: NativeFunction = native_fn(|this, _| Ok(this.get().clone()));
