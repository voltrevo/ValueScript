use std::rc::Rc;

use crate::{
  native_function::NativeFunction,
  vs_array::VsArray,
  vs_class::VsClass,
  vs_object::VsObject,
  vs_value::{LoadFunctionResult, Val, VsType},
  ValTrait,
};

pub struct StringBuiltin {}

pub static STRING_BUILTIN: StringBuiltin = StringBuiltin {};

impl ValTrait for StringBuiltin {
  fn typeof_(&self) -> VsType {
    VsType::Object
  }
  fn val_to_string(&self) -> String {
    "function String() { [native code] }".to_string()
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
    Val::String(Rc::new("function String() { [native code] }".to_string()))
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
  fn as_array_data(&self) -> Option<Rc<VsArray>> {
    None
  }
  fn as_object_data(&self) -> Option<Rc<VsObject>> {
    None
  }
  fn as_class_data(&self) -> Option<Rc<VsClass>> {
    None
  }

  fn load_function(&self) -> LoadFunctionResult {
    LoadFunctionResult::NotAFunction // TODO: Converts input to string
  }

  fn sub(&self, key: Val) -> Val {
    match key.val_to_string().as_str() {
      "fromCodePoint" => Val::Static(&FROM_CODE_POINT),
      _ => Val::Undefined,
    }
  }

  fn submov(&mut self, _key: Val, _value: Val) {
    std::panic!("Not implemented: exceptions");
  }

  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "\x1b[36m[String]\x1b[39m")
  }

  fn codify(&self) -> String {
    "String".into()
  }
}

static FROM_CODE_POINT: NativeFunction = NativeFunction {
  fn_: |_this: &mut Val, params: Vec<Val>| -> Val {
    let mut result = String::new();

    for param in params {
      let code_point = param.to_number() as u32; // TODO: Check overflow behavior

      let char = match std::char::from_u32(code_point) {
        Some(c) => c,
        None => panic!("Not implemented: exceptions (RangeError: Invalid code point)"),
      };

      result.push(char);
    }

    Val::String(Rc::new(result))
  },
};