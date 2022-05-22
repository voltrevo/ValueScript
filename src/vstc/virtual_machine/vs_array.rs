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
      "entries" => Val::Static(&ENTRIES),
      "every" => Val::Static(&EVERY),
      "fill" => Val::Static(&FILL),
      "filter" => Val::Static(&FILTER),
      "find" => Val::Static(&FIND),
      "findIndex" => Val::Static(&FIND_INDEX),
      "flat" => Val::Static(&FLAT),
      "flatMap" => Val::Static(&FLAT_MAP),
      // forEach: Not included because it cannot work as expected in ValueScript
      // (Use a for..of loop)
      "includes" => Val::Static(&INCLUDES),
      "indexOf" => Val::Static(&INDEX_OF),
      "join" => Val::Static(&JOIN),
      "keys" => Val::Static(&KEYS),
      "lastIndexOf" => Val::Static(&LAST_INDEX_OF),
      "map" => Val::Static(&MAP),
      "pop" => Val::Static(&POP),
      "push" => Val::Static(&PUSH),
      "reduce" => Val::Static(&REDUCE),
      "reduceRight" => Val::Static(&REDUCE_RIGHT),
      "reverse" => Val::Static(&REVERSE),
      "shift" => Val::Static(&SHIFT),
      "slice" => Val::Static(&SLICE),
      "some" => Val::Static(&SOME),
      "sort" => Val::Static(&SORT),
      "splice" => Val::Static(&SPLICE),
      "toLocaleString" => Val::Static(&TO_LOCALE_STRING),
      "toString" => Val::Static(&TO_STRING),
      "unshift" => Val::Static(&UNSHIFT),
      "values" => Val::Static(&VALUES),
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

fn to_wrapping_index_clamped(index: &Val, len: usize) -> isize {
  let wrapping_index = to_unchecked_wrapping_index(index, len);

  if wrapping_index < 0 {
    return 0;
  }

  if wrapping_index > len as isize {
    // len-1 would be a mistake. The end of the array is a meaningful index even
    // though there is no data there.
    return len as isize;
  }

  return wrapping_index;
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
        let ulen = array_data_mut.elements.len();

        if ulen > isize::MAX as usize {
          std::panic!("Not implemented: array len exceeds isize");
        }

        let mut target = match params.get(0) {
          None => 0,
          Some(p) => to_wrapping_index_clamped(p, ulen),
        };

        let mut start = match params.get(1) {
          None => 0,
          Some(p) => to_wrapping_index_clamped(p, ulen),
        };

        let ilen = ulen as isize;

        let mut end = match params.get(2) {
          None => ilen,
          // FIXME: undefined -> len (and others in this file)
          Some(p) => to_wrapping_index_clamped(p, ulen),
        };

        let copy_len = end - start;

        if copy_len <= 0 {
          return this.clone();
        }

        if target <= start || target >= end {
          while target < ilen && start < end {
            array_data_mut.elements[target as usize] =
              array_data_mut.elements[start as usize].clone()
            ;

            target += 1;
            start += 1;
          }
        } else {
          // The target is after the start. If we copied from start to target
          // and worked forwards we'd overwrite the values we needed later.
          // Instead we simply do the copies in the reverse order.

          target += copy_len - 1;
          end -= 1;

          if target >= ilen {
            end -= target - ilen + 1;
            target = ilen - 1;
          }

          while target >= 0 && end >= start {
            array_data_mut.elements[target as usize] =
              array_data_mut.elements[end as usize].clone()
            ;

            target -= 1;
            end -= 1;
          }
        }

        return this.clone();
      },
      _ => std::panic!("Not implemented: exceptions/array indirection")
    };
  }
};

static ENTRIES: NativeFunction = NativeFunction {
  fn_: |this: &mut Val, _params: Vec<Val>| -> Val {
    match this {
      Val::Array(_array_data) => {
        std::panic!("Not implemented: ENTRIES");
      },
      _ => std::panic!("Not implemented: exceptions/array indirection"),
    };
  }
};

static EVERY: NativeFunction = NativeFunction {
  fn_: |this: &mut Val, _params: Vec<Val>| -> Val {
    match this {
      Val::Array(_array_data) => {
        std::panic!("Not implemented: EVERY");
      },
      _ => std::panic!("Not implemented: exceptions/array indirection"),
    };
  }
};

static FILL: NativeFunction = NativeFunction {
  fn_: |this: &mut Val, params: Vec<Val>| -> Val {
    match this {
      Val::Array(array_data) => {
        let array_data_mut = Rc::make_mut(array_data);
        let len = array_data_mut.elements.len();

        let fill_val = params.get(0).unwrap_or(&Val::Undefined);

        let start = match params.get(1) {
          None => 0,
          Some(v) => to_wrapping_index_clamped(v, len),
        };

        let end = match params.get(2) {
          None => len as isize,
          Some(v) => to_wrapping_index_clamped(v, len),
        };

        for i in start..end {
          array_data_mut.elements[i as usize] = fill_val.clone();
        }

        return this.clone();
      },
      _ => std::panic!("Not implemented: exceptions/array indirection")
    };
  }
};

static FILTER: NativeFunction = NativeFunction {
  fn_: |this: &mut Val, _params: Vec<Val>| -> Val {
    match this {
      Val::Array(_array_data) => {
        std::panic!("Not implemented: FILTER");
      },
      _ => std::panic!("Not implemented: exceptions/array indirection"),
    };
  }
};

static FIND: NativeFunction = NativeFunction {
  fn_: |this: &mut Val, _params: Vec<Val>| -> Val {
    match this {
      Val::Array(_array_data) => {
        std::panic!("Not implemented: FIND");
      },
      _ => std::panic!("Not implemented: exceptions/array indirection"),
    };
  }
};

static FIND_INDEX: NativeFunction = NativeFunction {
  fn_: |this: &mut Val, _params: Vec<Val>| -> Val {
    match this {
      Val::Array(_array_data) => {
        std::panic!("Not implemented: FIND_INDEX");
      },
      _ => std::panic!("Not implemented: exceptions/array indirection"),
    };
  }
};

static FLAT: NativeFunction = NativeFunction {
  fn_: |this: &mut Val, params: Vec<Val>| -> Val {
    match this {
      Val::Array(array_data) => {
        if params.len() > 0 {
          std::panic!("Not implemented: .flat depth parameter");
        }

        let mut new_elems = Vec::<Val>::new();

        for el in &array_data.elements {
          match &el.as_array_data() {
            None => {
              new_elems.push(el.clone());
            },
            Some(p_array_data) => {
              for elem in &p_array_data.elements {
                new_elems.push(elem.clone());
              }
            },
          }
        }

        return Val::Array(Rc::new(VsArray::from(new_elems)));
      },
      _ => std::panic!("Not implemented: exceptions/array indirection")
    };
  }
};

static FLAT_MAP: NativeFunction = NativeFunction {
  fn_: |this: &mut Val, _params: Vec<Val>| -> Val {
    match this {
      Val::Array(_array_data) => {
        std::panic!("Not implemented: FLAT_MAP");
      },
      _ => std::panic!("Not implemented: exceptions/array indirection"),
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

static INDEX_OF: NativeFunction = NativeFunction {
  fn_: |this: &mut Val, _params: Vec<Val>| -> Val {
    match this {
      Val::Array(_array_data) => {
        std::panic!("Not implemented: INDEX_OF");
      },
      _ => std::panic!("Not implemented: exceptions/array indirection"),
    };
  }
};

static JOIN: NativeFunction = NativeFunction {
  fn_: |this: &mut Val, _params: Vec<Val>| -> Val {
    match this {
      Val::Array(_array_data) => {
        std::panic!("Not implemented: JOIN");
      },
      _ => std::panic!("Not implemented: exceptions/array indirection"),
    };
  }
};

static KEYS: NativeFunction = NativeFunction {
  fn_: |this: &mut Val, _params: Vec<Val>| -> Val {
    match this {
      Val::Array(_array_data) => {
        std::panic!("Not implemented: KEYS");
      },
      _ => std::panic!("Not implemented: exceptions/array indirection"),
    };
  }
};

static LAST_INDEX_OF: NativeFunction = NativeFunction {
  fn_: |this: &mut Val, _params: Vec<Val>| -> Val {
    match this {
      Val::Array(_array_data) => {
        std::panic!("Not implemented: LAST_INDEX_OF");
      },
      _ => std::panic!("Not implemented: exceptions/array indirection"),
    };
  }
};

static MAP: NativeFunction = NativeFunction {
  fn_: |this: &mut Val, _params: Vec<Val>| -> Val {
    match this {
      Val::Array(_array_data) => {
        std::panic!("Not implemented: MAP");
      },
      _ => std::panic!("Not implemented: exceptions/array indirection"),
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

static REDUCE: NativeFunction = NativeFunction {
  fn_: |this: &mut Val, _params: Vec<Val>| -> Val {
    match this {
      Val::Array(_array_data) => {
        std::panic!("Not implemented: REDUCE");
      },
      _ => std::panic!("Not implemented: exceptions/array indirection"),
    };
  }
};

static REDUCE_RIGHT: NativeFunction = NativeFunction {
  fn_: |this: &mut Val, _params: Vec<Val>| -> Val {
    match this {
      Val::Array(_array_data) => {
        std::panic!("Not implemented: REDUCE_RIGHT");
      },
      _ => std::panic!("Not implemented: exceptions/array indirection"),
    };
  }
};

static REVERSE: NativeFunction = NativeFunction {
  fn_: |this: &mut Val, _params: Vec<Val>| -> Val {
    match this {
      Val::Array(_array_data) => {
        std::panic!("Not implemented: REVERSE");
      },
      _ => std::panic!("Not implemented: exceptions/array indirection"),
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

static SLICE: NativeFunction = NativeFunction {
  fn_: |this: &mut Val, _params: Vec<Val>| -> Val {
    match this {
      Val::Array(_array_data) => {
        std::panic!("Not implemented: SLICE");
      },
      _ => std::panic!("Not implemented: exceptions/array indirection"),
    };
  }
};

static SOME: NativeFunction = NativeFunction {
  fn_: |this: &mut Val, _params: Vec<Val>| -> Val {
    match this {
      Val::Array(_array_data) => {
        std::panic!("Not implemented: SOME");
      },
      _ => std::panic!("Not implemented: exceptions/array indirection"),
    };
  }
};

static SORT: NativeFunction = NativeFunction {
  fn_: |this: &mut Val, _params: Vec<Val>| -> Val {
    match this {
      Val::Array(_array_data) => {
        std::panic!("Not implemented: SORT");
      },
      _ => std::panic!("Not implemented: exceptions/array indirection"),
    };
  }
};

static SPLICE: NativeFunction = NativeFunction {
  fn_: |this: &mut Val, _params: Vec<Val>| -> Val {
    match this {
      Val::Array(_array_data) => {
        std::panic!("Not implemented: SPLICE");
      },
      _ => std::panic!("Not implemented: exceptions/array indirection"),
    };
  }
};

static TO_LOCALE_STRING: NativeFunction = NativeFunction {
  fn_: |this: &mut Val, _params: Vec<Val>| -> Val {
    match this {
      Val::Array(_array_data) => {
        std::panic!("Not implemented: TO_LOCALE_STRING");
      },
      _ => std::panic!("Not implemented: exceptions/array indirection"),
    };
  }
};

static TO_STRING: NativeFunction = NativeFunction {
  fn_: |this: &mut Val, _params: Vec<Val>| -> Val {
    match this {
      Val::Array(_array_data) => {
        std::panic!("Not implemented: TO_STRING");
      },
      _ => std::panic!("Not implemented: exceptions/array indirection"),
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

static VALUES: NativeFunction = NativeFunction {
  fn_: |this: &mut Val, _params: Vec<Val>| -> Val {
    match this {
      Val::Array(_array_data) => {
        std::panic!("Not implemented: VALUES");
      },
      _ => std::panic!("Not implemented: exceptions/array indirection"),
    };
  }
};
