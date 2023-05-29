use std::{fmt, rc::Rc};

use crate::{
  native_function::ThisWrapper,
  vs_class::VsClass,
  vs_value::{LoadFunctionResult, Val},
  ValTrait,
};

use super::builtin_object::BuiltinObject;

pub struct BooleanBuiltin {}

impl BuiltinObject for BooleanBuiltin {
  fn bo_name() -> &'static str {
    "Boolean"
  }

  fn bo_sub(_key: &str) -> Val {
    Val::Undefined
  }

  fn bo_load_function() -> LoadFunctionResult {
    LoadFunctionResult::NativeFunction(|_: ThisWrapper, params: Vec<Val>| -> Result<Val, Val> {
      Ok(if let Some(value) = params.get(0) {
        Val::Bool(value.is_truthy())
      } else {
        Val::Bool(false)
      })
    })
  }

  fn bo_as_class_data() -> Option<Rc<VsClass>> {
    None
  }
}

impl fmt::Display for BooleanBuiltin {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "function Boolean() {{ [native code] }}")
  }
}
