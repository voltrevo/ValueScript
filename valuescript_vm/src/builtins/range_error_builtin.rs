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

pub struct RangeErrorBuiltin {}

impl BuiltinObject for RangeErrorBuiltin {
  fn bo_name() -> &'static str {
    "RangeError"
  }

  fn bo_sub(_key: &str) -> Val {
    Val::Undefined
  }

  fn bo_load_function() -> LoadFunctionResult {
    LoadFunctionResult::NativeFunction(to_range_error)
  }

  fn bo_as_class_data() -> Option<Rc<VsClass>> {
    Some(Rc::new(VsClass {
      constructor: Val::Static(&SET_MESSAGE),
      instance_prototype: make_range_error_prototype(),
    }))
  }
}

impl fmt::Display for RangeErrorBuiltin {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "function RangeError() {{ [native code] }}")
  }
}

// TODO: Static? (Rc -> Arc?)
fn make_range_error_prototype() -> Val {
  VsObject {
    string_map: BTreeMap::from([
      ("name".to_string(), "RangeError".to_val()),
      ("toString".to_string(), Val::Static(&RANGE_ERROR_TO_STRING)),
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

  op_submov(this.get_mut()?, "message".to_val(), message.to_val())?;

  Ok(Val::Undefined)
});

pub fn to_range_error(_: ThisWrapper, params: Vec<Val>) -> Result<Val, Val> {
  Ok(
    VsObject {
      string_map: BTreeMap::from([(
        "message".to_string(),
        match params.get(0) {
          Some(param) => param.clone().to_val_string(),
          None => "".to_val(),
        },
      )]),
      symbol_map: Default::default(),
      prototype: Some(make_range_error_prototype()),
    }
    .to_val(),
  )
}

static RANGE_ERROR_TO_STRING: NativeFunction = native_fn(|this, _params| {
  let message = this.get().sub("message".to_val())?;
  Ok(format!("RangeError({})", message).to_val())
});

pub trait ToRangeError {
  fn to_range_error(self) -> Val;
}

impl ToRangeError for &str {
  fn to_range_error(self) -> Val {
    self.to_string().to_range_error()
  }
}

impl ToRangeError for String {
  fn to_range_error(self) -> Val {
    self.to_val().to_range_error()
  }
}

impl ToRangeError for Val {
  fn to_range_error(self) -> Val {
    VsObject {
      string_map: BTreeMap::from([("message".to_string(), self)]),
      symbol_map: Default::default(),
      prototype: Some(make_range_error_prototype()),
    }
    .to_val()
  }
}
