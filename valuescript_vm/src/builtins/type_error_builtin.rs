use std::{collections::BTreeMap, rc::Rc};

use num_bigint::BigInt;

use crate::type_error;
use crate::{
  format_val,
  native_function::NativeFunction,
  operations::{op_sub, op_submov},
  vs_array::VsArray,
  vs_class::VsClass,
  vs_object::VsObject,
  vs_value::{LoadFunctionResult, Val, VsType},
  ValTrait,
};

pub struct TypeErrorBuiltin {}

pub static TYPE_ERROR_BUILTIN: TypeErrorBuiltin = TypeErrorBuiltin {};

impl ValTrait for TypeErrorBuiltin {
  fn typeof_(&self) -> VsType {
    VsType::Object
  }
  fn val_to_string(&self) -> String {
    "function TypeError() { [native code] }".to_string()
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
    Val::String(Rc::new(
      "function TypeError() { [native code] }".to_string(),
    ))
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
      instance_prototype: make_type_error_prototype(),
    }))
  }

  fn load_function(&self) -> LoadFunctionResult {
    LoadFunctionResult::NativeFunction(to_type_error)
  }

  fn sub(&self, _key: Val) -> Result<Val, Val> {
    Ok(Val::Undefined)
  }

  fn submov(&mut self, _key: Val, _value: Val) -> Result<(), Val> {
    type_error!("Cannot assign to subscript of TypeError builtin")
  }

  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "\x1b[36m[TypeError]\x1b[39m")
  }

  fn codify(&self) -> String {
    "TypeError".into()
  }
}

pub fn to_type_error(_: &mut Val, params: Vec<Val>) -> Result<Val, Val> {
  Ok(Val::Object(Rc::new(VsObject {
    string_map: BTreeMap::from([(
      "message".to_string(),
      Val::String(Rc::new(match params.get(0) {
        Some(param) => param.val_to_string(),
        None => "".to_string(),
      })),
    )]),
    prototype: Some(make_type_error_prototype()),
  })))
}

// TODO: Static? (Rc -> Arc?)
fn make_type_error_prototype() -> Val {
  Val::Object(Rc::new(VsObject {
    string_map: BTreeMap::from([
      (
        "name".to_string(),
        Val::String(Rc::new("TypeError".to_string())),
      ),
      ("toString".to_string(), Val::Static(&TYPE_ERROR_TO_STRING)),
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

static TYPE_ERROR_TO_STRING: NativeFunction = NativeFunction {
  fn_: |this: &mut Val, _params: Vec<Val>| -> Result<Val, Val> {
    let message = op_sub(this.clone(), format_val!("message"))?;
    Ok(format_val!("TypeError({})", message))
  },
};

#[macro_export]
macro_rules! type_error {
  ($fmt:expr $(, $($arg:expr),*)?) => {{
    let mut this = Val::Undefined;
    let formatted_string = format!($fmt $(, $($arg),*)?);
    Err(to_type_error(
      &mut this, vec![Val::String(Rc::new(formatted_string))]
    ).unwrap())
  }};
}
