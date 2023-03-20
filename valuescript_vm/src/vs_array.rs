use std::cmp::{max, min};
use std::rc::Rc;

use num_bigint::BigInt;

use crate::helpers::{to_wrapping_index, to_wrapping_index_clamped};

use super::array_higher_functions::array_every::EVERY;
use super::array_higher_functions::array_filter::FILTER;
use super::array_higher_functions::array_find::FIND;
use super::array_higher_functions::array_find_index::FIND_INDEX;
use super::array_higher_functions::array_flat_map::FLAT_MAP;
use super::array_higher_functions::array_map::MAP;
use super::array_higher_functions::array_reduce::REDUCE;
use super::array_higher_functions::array_reduce_right::REDUCE_RIGHT;
use super::array_higher_functions::array_some::SOME;
use super::array_higher_functions::array_sort::SORT;
use super::native_function::NativeFunction;
use super::operations::op_triple_eq_impl;
use super::vs_class::VsClass;
use super::vs_object::VsObject;
use super::vs_value::{LoadFunctionResult, Val, ValTrait, VsType};

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

  pub fn new() -> VsArray {
    return VsArray {
      elements: vec![],
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
  fn typeof_(&self) -> VsType {
    VsType::Object
  }
  fn val_to_string(&self) -> String {
    "".to_string()
  }
  fn to_number(&self) -> f64 {
    0_f64
  }
  fn to_index(&self) -> Option<usize> {
    None
  }
  fn is_primitive(&self) -> bool {
    false
  }
  fn to_primitive(&self) -> Val {
    Val::String(Rc::new("".to_string()))
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
    None
  }

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
    std::panic!("TODO: Exceptions");
  }

  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "\x1b[36m[Array Prototype]\x1b[39m")
  }

  fn codify(&self) -> String {
    "Array.prototype".into()
  }
}

static AT: NativeFunction = NativeFunction {
  fn_: |this: &mut Val, params: Vec<Val>| -> Val {
    match this {
      Val::Array(array_data) => match to_wrapping_index(params.get(0), array_data.elements.len()) {
        None => Val::Undefined,
        Some(i) => array_data.elements[i].clone(),
      },
      _ => std::panic!("TODO: Exceptions/array indirection"),
    }
  },
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
            }
            Some(p_array_data) => {
              for elem in &p_array_data.elements {
                new_array.elements.push(elem.clone());
              }
            }
          }
        }

        return Val::Array(Rc::new(new_array));
      }
      _ => std::panic!("TODO: Exceptions/array indirection"),
    };
  },
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
              array_data_mut.elements[start as usize].clone();

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
              array_data_mut.elements[end as usize].clone();

            target -= 1;
            end -= 1;
          }
        }

        return this.clone();
      }
      _ => std::panic!("TODO: Exceptions/array indirection"),
    };
  },
};

static ENTRIES: NativeFunction = NativeFunction {
  fn_: |this: &mut Val, _params: Vec<Val>| -> Val {
    match this {
      Val::Array(_array_data) => {
        std::panic!("Not implemented: ENTRIES");
      }
      _ => std::panic!("TODO: Exceptions/array indirection"),
    };
  },
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
      }
      _ => std::panic!("TODO: Exceptions/array indirection"),
    };
  },
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
            }
            Some(p_array_data) => {
              for elem in &p_array_data.elements {
                new_elems.push(elem.clone());
              }
            }
          }
        }

        return Val::Array(Rc::new(VsArray::from(new_elems)));
      }
      _ => std::panic!("TODO: Exceptions/array indirection"),
    };
  },
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
      }
      _ => std::panic!("TODO: Exceptions/array indirection"),
    };
  },
};

static INDEX_OF: NativeFunction = NativeFunction {
  fn_: |this: &mut Val, params: Vec<Val>| -> Val {
    match this {
      Val::Array(array_data) => {
        let search_param = params.get(0).unwrap_or(&Val::Undefined).clone();

        for i in 0..array_data.elements.len() {
          if op_triple_eq_impl(array_data.elements[i].clone(), search_param.clone()) {
            return Val::Number(i as f64);
          }
        }

        return Val::Number(-1_f64);
      }
      _ => std::panic!("TODO: Exceptions/array indirection"),
    };
  },
};

static JOIN: NativeFunction = NativeFunction {
  fn_: |this: &mut Val, params: Vec<Val>| -> Val {
    match this {
      Val::Array(vals) => {
        if vals.elements.len() == 0 {
          return Val::String(Rc::new("".to_string()));
        }

        if vals.elements.len() == 1 {
          return Val::String(Rc::new(vals.elements[0].val_to_string()));
        }

        let separator = params.get(0).unwrap_or(&Val::Undefined);

        let separator_str = match separator.typeof_() {
          VsType::Undefined => ",".to_string(),
          _ => separator.val_to_string(),
        };

        let mut iter = vals.elements.iter();
        let mut res = iter.next().unwrap().val_to_string();

        for val in iter {
          res += &separator_str;

          match val.typeof_() {
            VsType::Undefined => {}
            _ => {
              res += &val.val_to_string();
            }
          };
        }

        return Val::String(Rc::new(res));
      }
      _ => std::panic!("TODO: Exceptions/array indirection"),
    };
  },
};

static KEYS: NativeFunction = NativeFunction {
  fn_: |this: &mut Val, _params: Vec<Val>| -> Val {
    match this {
      Val::Array(_array_data) => {
        std::panic!("Not implemented: KEYS");
      }
      _ => std::panic!("TODO: Exceptions/array indirection"),
    };
  },
};

static LAST_INDEX_OF: NativeFunction = NativeFunction {
  fn_: |this: &mut Val, params: Vec<Val>| -> Val {
    match this {
      Val::Array(array_data) => {
        let search_param = params.get(0).unwrap_or(&Val::Undefined).clone();

        for i in (0..array_data.elements.len()).rev() {
          if op_triple_eq_impl(array_data.elements[i].clone(), search_param.clone()) {
            return Val::Number(i as f64);
          }
        }

        return Val::Number(-1_f64);
      }
      _ => std::panic!("TODO: Exceptions/array indirection"),
    };
  },
};

static POP: NativeFunction = NativeFunction {
  fn_: |this: &mut Val, _params: Vec<Val>| -> Val {
    match this {
      Val::Array(array_data) => {
        if array_data.elements.len() == 0 {
          return Val::Undefined;
        }

        let array_data_mut = Rc::make_mut(array_data);

        let removed_el = array_data_mut
          .elements
          .remove(array_data_mut.elements.len() - 1);

        return match removed_el {
          Val::Void => Val::Undefined,
          _ => removed_el,
        };
      }
      _ => std::panic!("TODO: Exceptions/array indirection"),
    };
  },
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
      }
      _ => std::panic!("TODO: Exceptions/array indirection"),
    };
  },
};

static REVERSE: NativeFunction = NativeFunction {
  fn_: |this: &mut Val, _params: Vec<Val>| -> Val {
    match this {
      Val::Array(array_data) => {
        if array_data.elements.len() == 0 {
          // Treating this as an edge case because rust protects us from
          // underflow when computing last below.
          return this.clone();
        }

        let array_data_mut = Rc::make_mut(array_data);

        let last = array_data_mut.elements.len() - 1;

        for i in 0..(array_data_mut.elements.len() / 2) {
          let tmp = array_data_mut.elements[i].clone();
          array_data_mut.elements[i] = array_data_mut.elements[last - i].clone();
          array_data_mut.elements[last - i] = tmp;
        }

        return this.clone();
      }
      _ => std::panic!("TODO: Exceptions/array indirection"),
    };
  },
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
      }
      _ => std::panic!("TODO: Exceptions/array indirection"),
    };
  },
};

static SLICE: NativeFunction = NativeFunction {
  fn_: |this: &mut Val, params: Vec<Val>| -> Val {
    match this {
      Val::Array(array_data) => {
        let mut new_elems = Vec::<Val>::new();

        let start = match params.get(0) {
          None => 0,
          Some(v) => to_wrapping_index_clamped(v, array_data.elements.len()),
        };

        let end = match params.get(1) {
          None => array_data.elements.len() as isize,
          Some(v) => to_wrapping_index_clamped(v, array_data.elements.len()),
        };

        for i in start..end {
          new_elems.push(array_data.elements[i as usize].clone());
        }

        return Val::Array(Rc::new(VsArray::from(new_elems)));
      }
      _ => std::panic!("TODO: Exceptions/array indirection"),
    };
  },
};

static SPLICE: NativeFunction = NativeFunction {
  fn_: |this: &mut Val, params: Vec<Val>| -> Val {
    match this {
      Val::Array(array_data) => {
        let array_data_mut = Rc::make_mut(array_data);
        let len = array_data_mut.elements.len();

        let start = match params.get(0) {
          None => 0,
          Some(v) => to_wrapping_index_clamped(v, len),
        } as usize;

        let delete_count_f64 = match params.get(1) {
          None => len as f64,
          Some(v) => match v.typeof_() {
            VsType::Undefined => len as f64,
            _ => v.to_number(),
          },
        };

        let delete_count = match delete_count_f64 < 0_f64 {
          true => 0,
          false => min(delete_count_f64.floor() as usize, len - start),
        };

        let mut deleted_elements = Vec::<Val>::new();

        for i in 0..delete_count {
          deleted_elements.push(array_data_mut.elements[start + i].clone());
        }

        let insert_len = max(2, params.len()) - 2;
        let replace_len = min(insert_len, delete_count);

        if insert_len > replace_len {
          for i in 0..replace_len {
            array_data_mut.elements[start + i] = params[i + 2].clone();
          }

          let gap = insert_len - replace_len;

          for _ in 0..gap {
            array_data_mut.elements.push(Val::Void);
          }

          for i in ((start + replace_len)..len).rev() {
            array_data_mut.elements[i + gap] = array_data_mut.elements[i].clone();
          }

          for i in replace_len..insert_len {
            array_data_mut.elements[start + i] = params[i + 2].clone();
          }
        } else {
          for i in 0..insert_len {
            array_data_mut.elements[start + i] = params[i + 2].clone();
          }

          let gap = delete_count - insert_len;

          if gap != 0 {
            for i in (start + insert_len)..(len - gap) {
              array_data_mut.elements[i] = array_data_mut.elements[i + gap].clone();
            }

            for _ in 0..gap {
              array_data_mut.elements.pop();
            }
          }
        }

        return Val::Array(Rc::new(VsArray::from(deleted_elements)));
      }
      _ => std::panic!("TODO: Exceptions/array indirection"),
    };
  },
};

static TO_LOCALE_STRING: NativeFunction = NativeFunction {
  fn_: |this: &mut Val, _params: Vec<Val>| -> Val {
    match this {
      Val::Array(_array_data) => {
        std::panic!("Not implemented: TO_LOCALE_STRING");
      }
      _ => std::panic!("TODO: Exceptions/array indirection"),
    };
  },
};

static TO_STRING: NativeFunction = NativeFunction {
  fn_: |this: &mut Val, _params: Vec<Val>| -> Val {
    return Val::String(Rc::new(this.val_to_string()));
  },
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
      }
      _ => std::panic!("TODO: Exceptions/array indirection"),
    };
  },
};

static VALUES: NativeFunction = NativeFunction {
  fn_: |this: &mut Val, _params: Vec<Val>| -> Val {
    match this {
      Val::Array(_array_data) => {
        std::panic!("Not implemented: VALUES");
      }
      _ => std::panic!("TODO: Exceptions/array indirection"),
    };
  },
};
