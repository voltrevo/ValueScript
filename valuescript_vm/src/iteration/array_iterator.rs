use std::{fmt, rc::Rc};

use num_bigint::BigInt;

use crate::{
  builtins::{internal_error_builtin::ToInternalError, type_error_builtin::ToTypeError},
  native_function::{native_fn, NativeFunction},
  vs_array::VsArray,
  vs_class::VsClass,
  vs_symbol::VsSymbol,
  vs_value::{dynamic_make_mut, ToDynamicVal, ToVal, Val, VsType},
  LoadFunctionResult, ValTrait,
};

use super::{
  iteration_result::IterationResult, iterator_has::iterator_has, return_this::RETURN_THIS,
};

#[derive(Clone)]
pub struct ArrayIterator {
  pub array: Rc<VsArray>,
  pub index: usize,
}

impl ArrayIterator {
  pub fn new(array: Rc<VsArray>) -> ArrayIterator {
    ArrayIterator { array, index: 0 }
  }
}

impl ValTrait for ArrayIterator {
  fn typeof_(&self) -> VsType {
    VsType::Object
  }

  fn to_number(&self) -> f64 {
    f64::NAN
  }

  fn to_index(&self) -> Option<usize> {
    None
  }

  fn is_primitive(&self) -> bool {
    false
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

  fn as_class_data(&self) -> Option<Rc<VsClass>> {
    None
  }

  fn load_function(&self) -> LoadFunctionResult {
    LoadFunctionResult::NotAFunction
  }

  fn sub(&self, key: &Val) -> Result<Val, Val> {
    if key.to_string() == "next" {
      return Ok(NEXT.to_val());
    }

    if let Val::Symbol(key) = key {
      match key {
        VsSymbol::ITERATOR => {
          return Ok(RETURN_THIS.to_val());
        }
      }
    }

    Ok(Val::Undefined)
  }

  fn has(&self, key: &Val) -> Option<bool> {
    iterator_has(key)
  }

  fn submov(&mut self, _key: &Val, _value: Val) -> Result<(), Val> {
    Err("Cannot assign to subscript of array iterator".to_type_error())
  }

  fn pretty_fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "\x1b[36m[ArrayIterator]\x1b[39m")
  }

  fn codify(&self) -> String {
    format!(
      "ArrayIterator({{ array: {}, index: {} }})",
      Val::Array(self.array.clone()).codify(),
      self.index
    )
  }
}

impl fmt::Display for ArrayIterator {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "[object Array Iterator]")
  }
}

static NEXT: NativeFunction = native_fn(|mut this, _| {
  let dynamic = match this.get_mut()? {
    Val::Dynamic(dynamic) => dynamic,
    _ => return Err("TODO: indirection".to_internal_error()),
  };

  let iter = dynamic_make_mut(dynamic)
    .as_any_mut()
    .downcast_mut::<ArrayIterator>()
    .ok_or_else(|| "ArrayIterator.next called on different object".to_type_error())?;

  match iter.array.elements.get(iter.index) {
    Some(item) => {
      iter.index += 1;

      Ok(
        IterationResult {
          value: item.clone(),
          done: false,
        }
        .to_dynamic_val(),
      )
    }
    None => Ok(
      IterationResult {
        value: Val::Undefined,
        done: true,
      }
      .to_dynamic_val(),
    ),
  }
});
