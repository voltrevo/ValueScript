use std::collections::BTreeMap;
use std::fmt;
use std::rc::Rc;

use crate::native_function::{native_fn, ThisWrapper};
use crate::vs_class::VsClass;
use crate::vs_value::ToVal;
use crate::ValTrait;
use crate::{
  native_function::NativeFunction,
  operations::op_submov,
  vs_object::VsObject,
  vs_value::{LoadFunctionResult, Val},
};

use super::builtin_object::BuiltinObject;

pub struct ErrorBuiltin {}

impl BuiltinObject for ErrorBuiltin {
  fn bo_name() -> &'static str {
    "Error"
  }

  fn bo_sub(_key: &str) -> Val {
    Val::Undefined
  }

  fn bo_load_function() -> LoadFunctionResult {
    LoadFunctionResult::NativeFunction(|_: ThisWrapper, params: Vec<Val>| -> Result<Val, Val> {
      Ok(
        match params.get(0) {
          Some(param) => param.clone().to_val_string(),
          None => "".to_val(),
        }
        .to_error(),
      )
    })
  }

  fn bo_as_class_data() -> Option<Rc<VsClass>> {
    Some(Rc::new(VsClass {
      constructor: SET_MESSAGE.to_val(),
      instance_prototype: make_error_prototype(),
    }))
  }
}

impl fmt::Display for ErrorBuiltin {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "function Error() {{ [native code] }}")
  }
}

pub trait ToError {
  fn to_error(self) -> Val;
}

impl ToError for Val {
  fn to_error(self) -> Val {
    VsObject {
      string_map: BTreeMap::from([("message".to_string(), self.to_val_string())]),
      symbol_map: Default::default(),
      prototype: Some(make_error_prototype()),
    }
    .to_val()
  }
}

impl ToError for String {
  fn to_error(self) -> Val {
    self.to_val().to_error()
  }
}

impl ToError for &str {
  fn to_error(self) -> Val {
    self.to_string().to_error()
  }
}

// TODO: Static? (Rc -> Arc?)
fn make_error_prototype() -> Val {
  VsObject {
    string_map: BTreeMap::from([
      ("name".to_string(), "Error".to_val()),
      ("toString".to_string(), ERROR_TO_STRING.to_val()),
    ]),
    symbol_map: Default::default(),
    prototype: None,
  }
  .to_val()
}

static SET_MESSAGE: NativeFunction = native_fn(|mut this, params| {
  let message = match params.get(0) {
    Some(param) => param.to_string(),
    None => "".to_string(),
  };

  op_submov(this.get_mut()?, &"message".to_val(), message.to_val())?;

  Ok(Val::Undefined)
});

static ERROR_TO_STRING: NativeFunction = native_fn(|this, _params| {
  let message = this.get().sub(&"message".to_val())?;
  Ok(format!("Error({})", message).to_val()) // TODO: Fixes needed here (and other errors)
});
