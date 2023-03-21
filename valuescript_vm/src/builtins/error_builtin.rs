use std::{collections::BTreeMap, rc::Rc};

use num_bigint::BigInt;

use crate::{
  format_err, format_val,
  native_function::NativeFunction,
  operations::{op_sub, op_submov},
  vs_array::VsArray,
  vs_class::VsClass,
  vs_object::VsObject,
  vs_value::{LoadFunctionResult, Val, VsType},
  ValTrait,
};

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
    Val::String(Rc::new("function Error() { [native code] }".to_string()))
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
    LoadFunctionResult::NativeFunction(to_error)
  }

  fn sub(&self, _key: Val) -> Result<Val, Val> {
    Ok(Val::Undefined)
  }

  fn submov(&mut self, _key: Val, _value: Val) -> Result<(), Val> {
    format_err!("TypeError: Cannot assign to subscript of Error builtin")
  }

  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "\x1b[36m[Error]\x1b[39m")
  }

  fn codify(&self) -> String {
    "Error".into()
  }
}

fn to_error(_: &mut Val, params: Vec<Val>) -> Result<Val, Val> {
  Ok(Val::Object(Rc::new(VsObject {
    string_map: BTreeMap::from([(
      "message".to_string(),
      Val::String(Rc::new(match params.get(0) {
        Some(param) => param.val_to_string(),
        None => "".to_string(),
      })),
    )]),
    prototype: Some(make_error_prototype()),
  })))
}

// TODO: Static? (Rc -> Arc?)
fn make_error_prototype() -> Val {
  Val::Object(Rc::new(VsObject {
    string_map: BTreeMap::from([
      (
        "name".to_string(),
        Val::String(Rc::new("Error".to_string())),
      ),
      ("toString".to_string(), Val::Static(&ERROR_TO_STRING)),
    ]),
    prototype: None,
  }))
}

static SET_MESSAGE: NativeFunction = NativeFunction {
  fn_: |this: &mut Val, params: Vec<Val>| -> Result<Val, Val> {
    let message = match params.get(0) {
      Some(param) => param.val_to_string(),
      None => "".to_string(),
    };

    op_submov(this, format_val!("message"), format_val!("{}", message))?;

    Ok(Val::Undefined)
  },
};

static ERROR_TO_STRING: NativeFunction = NativeFunction {
  fn_: |this: &mut Val, _params: Vec<Val>| -> Result<Val, Val> {
    let message = op_sub(this.clone(), format_val!("message"))?;
    Ok(format_val!("Error({})", message))
  },
};
