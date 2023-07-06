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
pub struct StringIterator {
  pub string: Rc<str>,
  pub index: usize,
}

impl StringIterator {
  pub fn new(string: Rc<str>) -> StringIterator {
    StringIterator { string, index: 0 }
  }

  fn next(&mut self) -> Option<char> {
    let bytes = self.string.as_bytes();

    if self.index >= bytes.len() {
      return None;
    }

    let byte = bytes[self.index];
    self.index += 1;

    let leading_ones = byte.leading_ones() as usize;

    if leading_ones == 0 {
      return Some(std::char::from_u32(byte as u32).expect("Invalid code point"));
    }

    if leading_ones == 1 || leading_ones > 4 || (self.index - 1) + leading_ones > bytes.len() {
      panic!("Invalid unicode");
    }

    let mut value = (byte & (0x7F >> leading_ones)) as u32;

    for _ in 1..leading_ones {
      let next_byte = bytes[self.index];
      self.index += 1;

      if next_byte.leading_ones() != 1 {
        return None;
      }

      value = (value << 6) | (next_byte & 0x3F) as u32;
    }

    Some(std::char::from_u32(value as u32).expect("Invalid code point"))
  }
}

impl ValTrait for StringIterator {
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
    Err("Cannot assign to subscript of string iterator".to_type_error())
  }

  fn pretty_fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "\x1b[36m[StringIterator]\x1b[39m")
  }

  fn codify(&self) -> String {
    format!(
      "StringIterator({{ string: {}, index: {} }})",
      Val::String(self.string.clone()).codify(),
      self.index
    )
  }
}

impl fmt::Display for StringIterator {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "[object String Iterator]")
  }
}

static NEXT: NativeFunction = native_fn(|mut this, _| {
  let dynamic = match this.get_mut()? {
    Val::Dynamic(dynamic) => dynamic,
    _ => return Err("TODO: indirection".to_internal_error()),
  };

  let iter = dynamic_make_mut(dynamic)
    .as_any_mut()
    .downcast_mut::<StringIterator>()
    .ok_or_else(|| "StringIterator.next called on different object".to_type_error())?;

  Ok(
    match iter.next() {
      Some(c) => IterationResult {
        value: c.to_val(),
        done: false,
      },
      None => IterationResult {
        value: Val::Undefined,
        done: true,
      },
    }
    .to_dynamic_val(),
  )
});
