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
      "at" => Val::Static(&AT),
      "concat" => Val::Static(&CONCAT),
      "push" => Val::Static(&PUSH),
      "unshift" => Val::Static(&UNSHIFT),
      "pop" => Val::Static(&POP),
      "shift" => Val::Static(&SHIFT),
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

static AT: NativeFunction = NativeFunction {
  fn_: |this: &mut Val, params: Vec<Val>| -> Val {
    match this {
      Val::Array(array_data) => {
        let index = match params.get(0) {
          None => 0_f64,
          Some(v) => v.to_number(),
        };

        let abs_index = index.abs();

        if abs_index == f64::INFINITY {
          return Val::Undefined;
        }

        let mut floored_index = index.signum() * abs_index.floor();

        let f64_len = array_data.elements.len() as f64;

        if floored_index < 0_f64 {
          floored_index += f64_len;
        }

        if floored_index < 0_f64 || floored_index >= f64_len {
          return Val::Undefined;
        }

        return array_data.elements[floored_index as usize].clone();
      },
      _ => std::panic!("Not implemented: exceptions/array indirection")
    };
  }
};

static CONCAT: NativeFunction = NativeFunction {
  fn_: |this: &mut Val, params: Vec<Val>| -> Val {
    match this {
      Val::Array(array_data) => {
        let mut new_array = array_data.as_ref().clone();

        for p in params {
          match &p.as_array_data() {
            None => {
              new_array.elements.push(p);
            },
            Some(p_array_data) => {
              for elem in &p_array_data.elements {
                new_array.elements.push(elem.clone());
              }
            },
          }
        }

        return Val::Array(Rc::new(new_array));
      },
      _ => std::panic!("Not implemented: exceptions/array indirection")
    };
  }
};

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

static UNSHIFT: NativeFunction = NativeFunction {
  fn_: |this: &mut Val, params: Vec<Val>| -> Val {
    match this {
      Val::Array(array_data) => {
        let array_data_mut = Rc::make_mut(array_data);

        let mut i = 0;

        for p in params {
          array_data_mut.elements.insert(i, p);
          i += 1;
        }

        return Val::Number(array_data_mut.elements.len() as f64);
      },
      _ => std::panic!("Not implemented: exceptions/array indirection")
    };
  }
};

static POP: NativeFunction = NativeFunction {
  fn_: |this: &mut Val, _params: Vec<Val>| -> Val {
    match this {
      Val::Array(array_data) => {
        if array_data.elements.len() == 0 {
          return Val::Undefined;
        }

        let array_data_mut = Rc::make_mut(array_data);

        return array_data_mut.elements.remove(array_data_mut.elements.len() - 1);
      },
      _ => std::panic!("Not implemented: exceptions/array indirection")
    };
  }
};

static SHIFT: NativeFunction = NativeFunction {
  fn_: |this: &mut Val, _params: Vec<Val>| -> Val {
    match this {
      Val::Array(array_data) => {
        if array_data.elements.len() == 0 {
          return Val::Undefined;
        }

        let array_data_mut = Rc::make_mut(array_data);

        return array_data_mut.elements.remove(0);
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
