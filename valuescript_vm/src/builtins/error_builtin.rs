use std::{collections::BTreeMap, rc::Rc};

use num_bigint::BigInt;

use crate::native_function::ThisWrapper;
use crate::vs_value::{ToVal, ToValString};
use crate::{
  native_function::NativeFunction,
  operations::{op_sub, op_submov},
  vs_array::VsArray,
  vs_class::VsClass,
  vs_object::VsObject,
  vs_value::{LoadFunctionResult, Val, VsType},
  ValTrait,
};

use super::type_error_builtin::ToTypeError;

pub struct ErrorBuiltin {}

pub static ERROR_BUILTIN: ErrorBuiltin = ErrorBuiltin {};

impl ValTrait for ErrorBuiltin {
  fn typeof_(&self) -> VsType {
    VsType::Object
  }
  fn val_to_string(&self) -> String {
    "function Error() { [native code] }".to_string()
  }
  fn to_number(&self) -> f64 {
    core::f64::NAN
  }
  fn to_index(&self) -> Option<usize> {
    None
  }
  fn is_primitive(&self) -> bool {
    false
  }
  fn to_primitive(&self) -> Val {
    self.to_val_string()
  }
  fn is_truthy(&self) -> bool {
    true
  }
  fn is_nullish(&self) -> bool {
    false
  }
  fn bind(&self, _params: Vec<Val>) -> Option<Val> {
    None
  }
  fn as_bigint_data(&self) -> Option<BigInt> {
    None
  }
  fn as_array_data(&self) -> Option<Rc<VsArray>> {
    None
  }
  fn as_object_data(&self) -> Option<Rc<VsObject>> {
    None
  }
  fn as_class_data(&self) -> Option<Rc<VsClass>> {
    Some(Rc::new(VsClass {
      constructor: Val::Static(&SET_MESSAGE),
      instance_prototype: make_error_prototype(),
    }))
  }

  fn load_function(&self) -> LoadFunctionResult {
    LoadFunctionResult::NativeFunction(|_: ThisWrapper, params: Vec<Val>| -> Result<Val, Val> {
      Ok(
        match params.get(0) {
          Some(param) => param.to_val_string(),
          None => "".to_val(),
        }
        .to_error(),
      )
    })
  }

  fn sub(&self, _key: Val) -> Result<Val, Val> {
    Ok(Val::Undefined)
  }

  fn submov(&mut self, _key: Val, _value: Val) -> Result<(), Val> {
    Err("Cannot assign to subscript of Error builtin".to_type_error())
  }

  fn next(&mut self) -> LoadFunctionResult {
    LoadFunctionResult::NotAFunction
  }

  fn pretty_fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "\x1b[36m[Error]\x1b[39m")
  }

  fn codify(&self) -> String {
    "Error".into()
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

static SET_MESSAGE: NativeFunction = NativeFunction {
  fn_: |mut this: ThisWrapper, params: Vec<Val>| -> Result<Val, Val> {
    let message = match params.get(0) {
      Some(param) => param.val_to_string(),
      None => "".to_string(),
    };

    op_submov(this.get_mut()?, "message".to_val(), message.to_val())?;

    Ok(Val::Undefined)
  },
};

static ERROR_TO_STRING: NativeFunction = NativeFunction {
  fn_: |this: ThisWrapper, _params: Vec<Val>| -> Result<Val, Val> {
    let message = op_sub(this.get().clone(), "message".to_val())?;
    Ok(format!("Error({})", message.val_to_string()).to_val()) // TODO: Fixes needed here (and other errors)
  },
};
