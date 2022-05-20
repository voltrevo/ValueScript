use std::rc::Rc;

use super::vs_value::{
  Val,
  VsType,
  ValTrait,
  LoadFunctionResult,
};
use super::vs_object::VsObject;
use super::native_function::NativeFunction;
use super::operations::op_triple_eq_impl;

#[derive(Clone)]
pub struct VsArray {
  pub elements: Vec<Val>,
  pub object: VsObject,
}

impl VsArray {
  pub fn from(vals: Vec<Val>) -> VsArray {
    return VsArray {
      elements: vals,
      object: VsObject {
        string_map: Default::default(),
        prototype: Some(Val::Static(&ARRAY_PROTOTYPE)),
      },
    };
  }
}

pub struct ArrayPrototype {}

static ARRAY_PROTOTYPE: ArrayPrototype = ArrayPrototype {};

impl ValTrait for ArrayPrototype {
  fn typeof_(&self) -> VsType { VsType::Object }
  fn val_to_string(&self) -> String { "".to_string() }
  fn to_number(&self) -> f64 { 0_f64 }
  fn to_index(&self) -> Option<usize> { None }
  fn is_primitive(&self) -> bool { false }
  fn to_primitive(&self) -> Val { Val::String(Rc::new("".to_string())) }
  fn is_truthy(&self) -> bool { true }
  fn is_nullish(&self) -> bool { false }

  fn bind(&self, _params: Vec<Val>) -> Option<Val> { None }

  fn as_array_data(&self) -> Option<Rc<VsArray>> { None }
  fn as_object_data(&self) -> Option<Rc<VsObject>> { None }

  fn load_function(&self) -> LoadFunctionResult {
    LoadFunctionResult::NotAFunction
  }

  fn sub(&self, key: Val) -> Val {
    match key.val_to_string().as_str() {
      "push" => Val::Static(&PUSH),
      "includes" => Val::Static(&INCLUDES),
      _ => Val::Undefined,
    }
  }

  fn submov(&mut self, _key: Val, _value: Val) {
    std::panic!("Not implemented: exceptions");
  }

  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "\x1b[36m[Array Prototype]\x1b[39m")
  }
}

static PUSH: NativeFunction = NativeFunction {
  fn_: |this: &mut Val, params: Vec<Val>| -> Val {
    match this {
      Val::Array(array_data) => {
        let array_data_mut = Rc::make_mut(array_data);

        for p in params {
          array_data_mut.elements.push(p);
        }

        return Val::Number(array_data_mut.elements.len() as f64);
      },
      _ => std::panic!("Not implemented: exceptions/array indirection")
    };
  }
};

static INCLUDES: NativeFunction = NativeFunction {
  fn_: |this: &mut Val, params: Vec<Val>| -> Val {
    match this {
      Val::Array(array_data) => {
        let search_param = params.get(0).unwrap_or(&Val::Undefined).clone();

        for elem in &array_data.elements {
          if op_triple_eq_impl(elem.clone(), search_param.clone()) {
            return Val::Bool(true);
          }
        }

        return Val::Bool(false);
      },
      _ => std::panic!("Not implemented: exceptions/array indirection")
    };
  }
};
