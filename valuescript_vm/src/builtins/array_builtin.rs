use std::{fmt, rc::Rc};

use crate::{
  native_function::{native_fn, NativeFunction, ThisWrapper},
  vs_array::VsArray,
  vs_class::VsClass,
  vs_value::{LoadFunctionResult, ToVal, Val},
  ValTrait,
};

use super::{
  builtin_object::BuiltinObject, internal_error_builtin::ToInternalError,
  range_error_builtin::ToRangeError, type_error_builtin::ToTypeError,
};

pub struct ArrayBuiltin {}

impl BuiltinObject for ArrayBuiltin {
  fn bo_name() -> &'static str {
    "Array"
  }

  fn bo_sub(key: &str) -> Val {
    match key {
      "isArray" => IS_ARRAY.to_val(),
      "from" => FROM.to_val(),
      "of" => OF.to_val(),
      _ => Val::Undefined,
    }
  }

  fn bo_load_function() -> LoadFunctionResult {
    LoadFunctionResult::NativeFunction(to_array)
  }

  fn bo_as_class_data() -> Option<Rc<VsClass>> {
    None
  }
}

impl fmt::Display for ArrayBuiltin {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "function Array() {{ [native code] }}")
  }
}

static IS_ARRAY: NativeFunction = native_fn(|_this, params| {
  Ok(match params.first() {
    None => Val::Bool(false),
    Some(p) => match p.as_array_data() {
      None => Val::Bool(false),
      Some(_) => Val::Bool(true),
    },
  })
});

static FROM: NativeFunction = native_fn(|_this, params| {
  let mut first_param = match params.first() {
    None => return Err("undefined is not iterable".to_type_error()),
    Some(p) => p.clone(),
  };

  if params.len() > 1 {
    return Err("TODO: Using Array.from with a map function".to_internal_error());
  }

  if let Val::StoragePtr(ptr) = first_param {
    first_param = ptr.get();
  }

  Ok(match first_param {
    Val::Array(arr) => Val::Array(arr.clone()),
    Val::String(s) => s.chars().map(|c| c.to_val()).collect::<Vec<Val>>().to_val(),
    Val::Void | Val::Undefined | Val::Null | Val::CopyCounter(..) => {
      return Err("items is not iterable".to_type_error())
    }
    Val::Bool(..) | Val::Number(..) | Val::BigInt(..) | Val::Symbol(..) => {
      VsArray::default().to_val()
    }
    Val::Object(..) | Val::Function(..) | Val::Class(..) | Val::Static(..) | Val::Dynamic(..) => {
      let len = first_param
        .sub(&"length".to_val())
        .map_err(|e| e.to_string())
        .unwrap() // TODO: Exception
        .to_number();

      if len.is_sign_negative() || len.is_nan() {
        return Ok(VsArray::default().to_val());
      }

      if len.is_infinite() {
        return Err("Invalid array length".to_range_error());
      }

      let len = len as usize;

      let mut arr = Vec::with_capacity(len);

      // TODO: We should probably use a frame and step through this
      for i in 0..len {
        arr.push(
          first_param
            .sub(&(i as f64).to_val())
            .map_err(|e| e.to_string())
            .unwrap(), // TODO: Exception
        );
      }

      VsArray::from(arr).to_val()
    }
    Val::StoragePtr(_) => {
      panic!("Shouldn't be possible") // prevented this just above the match
    }
  })
});

static OF: NativeFunction = native_fn(|_this, params| Ok(VsArray::from(params).to_val()));

fn to_array(_: ThisWrapper, params: Vec<Val>) -> Result<Val, Val> {
  if params.len() != 1 {
    return Ok(VsArray::from(params).to_val());
  }

  Ok(match params[0] {
    Val::Number(number) => {
      if number.is_sign_negative() || number != number.floor() {
        return Err("Invalid array length".to_range_error());
      }

      let len = number as usize;

      let mut arr = Vec::with_capacity(len);

      for _ in 0..len {
        arr.push(Val::Void);
      }

      VsArray::from(arr).to_val()
    }
    _ => VsArray::from(params).to_val(),
  })
}
