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
      "copyWithin" => Val::Static(&COPY_WITHIN),
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

fn to_unchecked_wrapping_index(index: &Val, len: usize) -> isize {
  let index_num = index.to_number();

  let abs_index = index_num.abs();
  let mut floored_index = index_num.signum() * abs_index.floor();
  let f64_len = len as f64;

  if floored_index < 0_f64 {
    floored_index += f64_len;
  }

  // TODO: Investigate potential pitfalls for arrays with length exceeding max
  // isize.
  return floored_index as isize;
}

fn to_wrapping_index(index: Option<&Val>, len: usize) -> Option<usize> {
  let unchecked = match index {
    None => { return None; }
    Some(i) => to_unchecked_wrapping_index(i, len),
  };

  if unchecked < 0 || unchecked as usize >= len {
    return None;
  }

  return Some(unchecked as usize);
}

static AT: NativeFunction = NativeFunction {
  fn_: |this: &mut Val, params: Vec<Val>| -> Val {
    match this {
      Val::Array(array_data) => match to_wrapping_index(
        params.get(0),
        array_data.elements.len(),
      ) {
        None => Val::Undefined,
        Some(i) => array_data.elements[i].clone(),
      },
      _ => std::panic!("Not implemented: exceptions/array indirection")
    }
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

static COPY_WITHIN: NativeFunction = NativeFunction {
  fn_: |this: &mut Val, params: Vec<Val>| -> Val {
    match this {
      Val::Array(array_data) => {
        let array_data_mut = Rc::make_mut(array_data);
        let len = array_data_mut.elements.len();

        let mut target = match params.get(0) {
          None => 0,
          Some(p) => to_unchecked_wrapping_index(p, len),
        };

        let mut start = match params.get(1) {
          None => 0,
          Some(p) => to_unchecked_wrapping_index(p, len),
        };

        let end = match params.get(2) {
          None => len as isize,
          Some(p) => to_unchecked_wrapping_index(p, len),
        };

        if target < 0 {
          start += -target;
          target = 0;
        }

        let copy_len = end - start;

        if copy_len <= 0 {
          return this.clone();
        }

        if target > start && target < end {
          // Tricky case - make sure we don't read from things we've written
          std::panic!("Not implemented");

          //  1, 2, 3, 4, 5, 6, 7
          //        ^--------^
          //           ^
        } else {
          // Easy case - no overlap between read and write
          while target < len as isize && start < end {
            array_data_mut.elements[target as usize] =
              array_data_mut.elements[start as usize].clone()
            ;

            target += 1;
            start += 1;
          }
        }
        
        return this.clone();
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
