use std::cmp::{max, min};
use std::mem::take;
use std::rc::Rc;

use crate::array_higher_functions::{
  array_every::EVERY, array_filter::FILTER, array_find::FIND, array_find_index::FIND_INDEX,
  array_flat_map::FLAT_MAP, array_map::MAP, array_reduce::REDUCE, array_reduce_right::REDUCE_RIGHT,
  array_some::SOME, array_sort::SORT,
};
use crate::builtins::internal_error_builtin::ToInternalError;
use crate::helpers::{to_wrapping_index, to_wrapping_index_clamped};
use crate::iteration::array_entries_iterator::ArrayEntriesIterator;
use crate::iteration::array_iterator::ArrayIterator;
use crate::native_function::{native_fn, NativeFunction};
use crate::operations::op_triple_eq_impl;
use crate::todo_fn::TODO;
use crate::vs_array::VsArray;
use crate::vs_symbol::VsSymbol;
use crate::vs_value::{ToDynamicVal, ToVal, Val, ValTrait, VsType};

pub fn op_sub_array(array: &mut Rc<VsArray>, key: &Val) -> Result<Val, Val> {
  if let Some(index) = key.to_index() {
    return op_sub_array_index(array, index);
  }

  if let Val::Symbol(symbol) = key {
    return Ok(
      match symbol {
        VsSymbol::ITERATOR => &VALUES,
      }
      .to_val(),
    );
  }

  Ok(Val::Static(match key.to_string().as_str() {
    "at" => &AT,
    "concat" => &CONCAT,
    "copyWithin" => &COPY_WITHIN,
    "entries" => &ENTRIES,
    "every" => &EVERY,
    "fill" => &FILL,
    "filter" => &FILTER,
    "find" => &FIND,
    "findIndex" => &FIND_INDEX,
    "flat" => &FLAT,
    "flatMap" => &FLAT_MAP,
    // forEach: Not included because it cannot work as expected in ValueScript
    // (Use a for..of loop)
    "includes" => &INCLUDES,
    "indexOf" => &INDEX_OF,
    "join" => &JOIN,
    "keys" => &TODO,
    "lastIndexOf" => &LAST_INDEX_OF,
    "length" => return Ok((array.elements.len() as f64).to_val()),
    "map" => &MAP,
    "pop" => &POP,
    "push" => &PUSH,
    "reduce" => &REDUCE,
    "reduceRight" => &REDUCE_RIGHT,
    "reverse" => &REVERSE,
    "shift" => &SHIFT,
    "slice" => &SLICE,
    "some" => &SOME,
    "sort" => &SORT,
    "splice" => &SPLICE,
    "toLocaleString" => &TODO,
    "toString" => &TO_STRING,
    "unshift" => &UNSHIFT,
    "values" => &VALUES,
    _ => return Ok(Val::Undefined),
  }))
}

pub fn op_sub_array_index(array: &mut Rc<VsArray>, index: usize) -> Result<Val, Val> {
  if index >= array.elements.len() {
    return Ok(Val::Undefined);
  }

  let res = match Rc::get_mut(array) {
    Some(array_mut) => take(&mut array_mut.elements[index]),
    None => array.elements[index].clone(),
  };

  Ok(match res {
    Val::Void => Val::Undefined,
    _ => res,
  })
}

static AT: NativeFunction = native_fn(|this, params| {
  Ok(match this.get() {
    Val::Array(array_data) => match to_wrapping_index(params.first(), array_data.elements.len()) {
      None => Val::Undefined,
      Some(i) => array_data.elements[i].clone(),
    },
    _ => return Err("array indirection".to_internal_error()),
  })
});

static CONCAT: NativeFunction = native_fn(|this, params| {
  Ok(match this.get() {
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

      new_array.to_val()
    }
    _ => return Err("array indirection".to_internal_error()),
  })
});

static COPY_WITHIN: NativeFunction = native_fn(|mut this, params| {
  let this = this.get_mut()?;

  Ok(match this {
    Val::Array(array_data) => {
      let array_data_mut = Rc::make_mut(array_data);
      let ulen = array_data_mut.elements.len();

      if ulen > isize::MAX as usize {
        return Err("TODO: array len exceeds isize".to_internal_error());
      }

      let mut target = match params.first() {
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
        return Ok(this.clone());
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
          array_data_mut.elements[target as usize] = array_data_mut.elements[end as usize].clone();

          target -= 1;
          end -= 1;
        }
      }

      this.clone()
    }
    _ => return Err("array indirection".to_internal_error()),
  })
});

static ENTRIES: NativeFunction = native_fn(|this, _params| match this.get() {
  Val::Array(array_data) => Ok(ArrayEntriesIterator::new(array_data.clone()).to_dynamic_val()),
  _ => Err("array indirection".to_internal_error()),
});

static FILL: NativeFunction = native_fn(|mut this, params| {
  let this = this.get_mut()?;

  Ok(match this {
    Val::Array(array_data) => {
      let array_data_mut = Rc::make_mut(array_data);
      let len = array_data_mut.elements.len();

      let fill_val = params.first().unwrap_or(&Val::Undefined);

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

      this.clone()
    }
    _ => return Err("array indirection".to_internal_error()),
  })
});

static FLAT: NativeFunction = native_fn(|this, params| {
  Ok(match this.get() {
    Val::Array(array_data) => {
      if !params.is_empty() {
        return Err("TODO: .flat depth parameter".to_internal_error());
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

      new_elems.to_val()
    }
    _ => return Err("array indirection".to_internal_error()),
  })
});

static INCLUDES: NativeFunction = native_fn(|this, params| {
  Ok(match this.get() {
    Val::Array(array_data) => {
      let search_param = params.first().unwrap_or(&Val::Undefined);

      for elem in &array_data.elements {
        let is_eq = op_triple_eq_impl(elem, search_param)
          .map_err(|e| e.to_string())
          .unwrap(); // TODO: Exception

        if is_eq {
          return Ok(Val::Bool(true));
        }
      }

      Val::Bool(false)
    }
    _ => return Err("array indirection".to_internal_error()),
  })
});

static INDEX_OF: NativeFunction = native_fn(|this, params| {
  Ok(match this.get() {
    Val::Array(array_data) => {
      let search_param = params.first().unwrap_or(&Val::Undefined);

      for i in 0..array_data.elements.len() {
        let is_eq = op_triple_eq_impl(&array_data.elements[i], search_param)
          .map_err(|e| e.to_string())
          .unwrap(); // TODO: Exception

        if is_eq {
          return Ok(Val::Number(i as f64));
        }
      }

      Val::Number(-1.0)
    }
    _ => return Err("array indirection".to_internal_error()),
  })
});

static JOIN: NativeFunction = native_fn(|this, params| {
  Ok(match this.get() {
    Val::Array(vals) => {
      if vals.elements.is_empty() {
        return Ok("".to_val());
      }

      if vals.elements.len() == 1 {
        return Ok(vals.elements[0].clone().to_val_string());
      }

      let separator = match params.first() {
        None => ",".to_string(),
        Some(v) => v.to_string(),
      };

      let mut iter = vals.elements.iter();
      let mut res = iter.next().unwrap().to_string();

      for val in iter {
        res += &separator;

        match val.typeof_() {
          VsType::Undefined => {}
          _ => {
            res += &val.to_string();
          }
        };
      }

      res.to_val()
    }
    _ => return Err("array indirection".to_internal_error()),
  })
});

static LAST_INDEX_OF: NativeFunction = native_fn(|this, params| {
  Ok(match this.get() {
    Val::Array(array_data) => {
      let search_param = params.first().unwrap_or(&Val::Undefined);

      for i in (0..array_data.elements.len()).rev() {
        let is_eq = op_triple_eq_impl(&array_data.elements[i], search_param)
          .map_err(|e| e.to_string())
          .unwrap(); // TODO: Exception

        if is_eq {
          return Ok(Val::Number(i as f64));
        }
      }

      Val::Number(-1_f64)
    }
    _ => return Err("array indirection".to_internal_error()),
  })
});

static POP: NativeFunction = native_fn(|mut this, _params| {
  let this = this.get_mut()?;

  Ok(match this {
    Val::Array(array_data) => {
      if array_data.elements.is_empty() {
        return Ok(Val::Undefined);
      }

      let array_data_mut = Rc::make_mut(array_data);

      let removed_el = array_data_mut
        .elements
        .remove(array_data_mut.elements.len() - 1);

      match removed_el {
        Val::Void => Val::Undefined,
        _ => removed_el,
      }
    }
    _ => return Err("array indirection".to_internal_error()),
  })
});

static PUSH: NativeFunction = native_fn(|mut this, mut params| {
  let this = this.get_mut()?;

  Ok(match this {
    Val::Array(array_data) => {
      let array_data_mut = Rc::make_mut(array_data);
      array_data_mut.elements.append(&mut params);
      (array_data_mut.elements.len() as f64).to_val()
    }
    _ => return Err("array indirection".to_internal_error()),
  })
});

static REVERSE: NativeFunction = native_fn(|mut this, _params| {
  let this = this.get_mut()?;

  Ok(match this {
    Val::Array(array_data) => {
      if array_data.elements.is_empty() {
        // Treating this as an edge case because rust protects us from
        // underflow when computing last below.
        return Ok(this.clone());
      }

      let array_data_mut = Rc::make_mut(array_data);

      let last = array_data_mut.elements.len() - 1;

      for i in 0..(array_data_mut.elements.len() / 2) {
        let tmp = array_data_mut.elements[i].clone();
        array_data_mut.elements[i] = array_data_mut.elements[last - i].clone();
        array_data_mut.elements[last - i] = tmp;
      }

      this.clone()
    }
    _ => return Err("array indirection".to_internal_error()),
  })
});

static SHIFT: NativeFunction = native_fn(|mut this, _params| {
  let this = this.get_mut()?;

  Ok(match this {
    Val::Array(array_data) => {
      if array_data.elements.is_empty() {
        return Ok(Val::Undefined);
      }

      let array_data_mut = Rc::make_mut(array_data);

      array_data_mut.elements.remove(0)
    }
    _ => return Err("array indirection".to_internal_error()),
  })
});

static SLICE: NativeFunction = native_fn(|this, params| {
  Ok(match this.get() {
    Val::Array(array_data) => {
      let mut new_elems = Vec::<Val>::new();

      let start = match params.first() {
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

      new_elems.to_val()
    }
    _ => return Err("array indirection".to_internal_error()),
  })
});

static SPLICE: NativeFunction = native_fn(|mut this, params| {
  let this = this.get_mut()?;

  Ok(match this {
    Val::Array(array_data) => {
      let array_data_mut = Rc::make_mut(array_data);
      let len = array_data_mut.elements.len();

      let start = match params.first() {
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
        array_data_mut.elements[start..(replace_len + start)]
          .clone_from_slice(&params[2..(replace_len + 2)]);

        let gap = insert_len - replace_len;

        for _ in 0..gap {
          array_data_mut.elements.push(Val::Void);
        }

        for i in ((start + replace_len)..len).rev() {
          array_data_mut.elements[i + gap] = array_data_mut.elements[i].clone();
        }

        array_data_mut.elements[(replace_len + start)..(insert_len + start)]
          .clone_from_slice(&params[(replace_len + 2)..(insert_len + 2)]);
      } else {
        array_data_mut.elements[start..(insert_len + start)]
          .clone_from_slice(&params[2..(insert_len + 2)]);

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

      deleted_elements.to_val()
    }
    _ => return Err("array indirection".to_internal_error()),
  })
});

// TODO: Share this? (JS doesn't?)
static TO_STRING: NativeFunction = native_fn(|this, _params| Ok(this.get().to_string().to_val()));

static UNSHIFT: NativeFunction = native_fn(|mut this, params| {
  let this = this.get_mut()?;

  Ok(match this {
    Val::Array(array_data) => {
      let array_data_mut = Rc::make_mut(array_data);

      for (i, p) in params.into_iter().enumerate() {
        array_data_mut.elements.insert(i, p);
      }

      Val::Number(array_data_mut.elements.len() as f64)
    }
    _ => return Err("array indirection".to_internal_error()),
  })
});

static VALUES: NativeFunction = native_fn(|this, _params| match this.get() {
  Val::Array(array_data) => Ok(ArrayIterator::new(array_data.clone()).to_dynamic_val()),
  _ => Err("array indirection".to_internal_error()),
});
