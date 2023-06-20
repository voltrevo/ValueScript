use std::fmt;
use std::{collections::BTreeMap, rc::Rc};

use crate::native_function::{native_fn, ThisWrapper};
use crate::vs_value::ToVal;
use crate::ValTrait;
use crate::{
  native_function::NativeFunction,
  operations::op_submov,
  vs_class::VsClass,
  vs_object::VsObject,
  vs_value::{LoadFunctionResult, Val},
};

use super::builtin_object::BuiltinObject;

pub struct TypeErrorBuiltin {}

impl BuiltinObject for TypeErrorBuiltin {
  fn bo_name() -> &'static str {
    "TypeError"
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
        .to_type_error(),
      )
    })
  }

  fn bo_as_class_data() -> Option<Rc<VsClass>> {
    Some(Rc::new(VsClass {
      constructor: Val::Static(&SET_MESSAGE),
      instance_prototype: make_type_error_prototype(),
    }))
  }
}

impl fmt::Display for TypeErrorBuiltin {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "function TypeError() {{ [native code] }}")
  }
}

// TODO: Static? (Rc -> Arc?)
fn make_type_error_prototype() -> Val {
  VsObject {
    string_map: BTreeMap::from([
      ("name".to_string(), "TypeError".to_val()),
      ("toString".to_string(), TYPE_ERROR_TO_STRING.to_val()),
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

static TYPE_ERROR_TO_STRING: NativeFunction = native_fn(|this, _params| {
  let message = this.get().sub(&"message".to_val())?;
  Ok(format!("TypeError({})", message).to_val())
});

pub trait ToTypeError {
  fn to_type_error(self) -> Val;
}

impl ToTypeError for &str {
  fn to_type_error(self) -> Val {
    self.to_string().to_type_error()
  }
}

impl ToTypeError for String {
  fn to_type_error(self) -> Val {
    self.to_val().to_type_error()
  }
}

impl ToTypeError for Val {
  fn to_type_error(self) -> Val {
    VsObject {
      string_map: BTreeMap::from([("message".to_string(), self)]),
      symbol_map: Default::default(),
      prototype: Some(make_type_error_prototype()),
    }
    .to_val()
  }
}
