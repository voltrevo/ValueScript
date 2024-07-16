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
        match params.first() {
          Some(param) => param.clone().to_val_string(),
          None => "".to_val(),
        }
        .to_type_error(),
      )
    })
  }

  fn bo_as_class_data() -> Option<Rc<VsClass>> {
    Some(Rc::new(VsClass {
      name: "TypeError".to_string(),
      content_hash: None,
      constructor: Val::Static(&SET_MESSAGE),
      prototype: make_type_error_prototype(),
      static_: VsObject::default().to_val(),
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
    prototype: Val::Void,
  }
  .to_val()
}

static SET_MESSAGE: NativeFunction = native_fn(|mut this, params| {
  let message = match params.first() {
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
      prototype: make_type_error_prototype(),
    }
    .to_val()
  }
}
