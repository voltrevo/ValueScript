use std::{fmt, rc::Rc};

use num_bigint::BigInt;

use crate::{
  builtins::range_error_builtin::to_range_error,
  native_function::{NativeFunction, ThisWrapper},
  operations::op_sub,
  range_error,
  vs_array::VsArray,
  vs_class::VsClass,
  vs_object::VsObject,
  vs_value::{LoadFunctionResult, ToVal, ToValString, Val, VsType},
  ValTrait,
};

use super::type_error_builtin::ToTypeError;

pub struct ArrayBuiltin {}

pub static ARRAY_BUILTIN: ArrayBuiltin = ArrayBuiltin {};

impl ValTrait for ArrayBuiltin {
  fn typeof_(&self) -> VsType {
    VsType::Object
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
    self.to_val_string()
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
    LoadFunctionResult::NativeFunction(to_array)
  }

  fn sub(&self, key: Val) -> Result<Val, Val> {
    Ok(Val::Static(match key.to_string().as_str() {
      "isArray" => &IS_ARRAY,
      "from" => &FROM,
      "of" => &OF,
      _ => return Ok(Val::Undefined),
    }))
  }

  fn submov(&mut self, _key: Val, _value: Val) -> Result<(), Val> {
    Err("Cannot assign to subscript of Array builtin".to_type_error())
  }

  fn next(&mut self) -> LoadFunctionResult {
    LoadFunctionResult::NotAFunction
  }

  fn pretty_fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "\x1b[36m[Array]\x1b[39m")
  }

  fn codify(&self) -> String {
    "Array".into()
  }
}

impl fmt::Display for ArrayBuiltin {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "function Array() {{ [native code] }}")
  }
}

static IS_ARRAY: NativeFunction = NativeFunction {
  fn_: |_this: ThisWrapper, params: Vec<Val>| -> Result<Val, Val> {
    Ok(match params.get(0) {
      None => Val::Bool(false),
      Some(p) => match p.as_array_data() {
        None => Val::Bool(false),
        Some(_) => Val::Bool(true),
      },
    })
  },
};

static FROM: NativeFunction = NativeFunction {
  fn_: |_this: ThisWrapper, params: Vec<Val>| -> Result<Val, Val> {
    let first_param = match params.get(0) {
      None => return Err("undefined is not iterable".to_type_error()),
      Some(p) => p,
    };

    if params.len() > 1 {
      return Err(format!("TODO: Using Array.from with a map function").to_val());
    }

    Ok(match first_param {
      Val::Array(arr) => Val::Array(arr.clone()),
      Val::String(s) => s.chars().map(|c| c.to_val()).collect::<Vec<Val>>().to_val(),
      Val::Void | Val::Undefined | Val::Null => {
        return Err("items is not iterable".to_type_error())
      }
      Val::Bool(..) | Val::Number(..) | Val::BigInt(..) | Val::Symbol(..) => {
        VsArray::new().to_val()
      }
      Val::Object(..) | Val::Function(..) | Val::Class(..) | Val::Static(..) | Val::Custom(..) => {
        let len = op_sub(first_param.clone(), "length".to_val())
          .map_err(|e| e.to_string())
          .unwrap() // TODO: Exception
          .to_number();

        if len.is_sign_negative() || len.is_nan() {
          return Ok(VsArray::new().to_val());
        }

        if len.is_infinite() {
          return range_error!("Invalid array length");
        }

        let len = len as usize;

        let mut arr = Vec::with_capacity(len);

        // TODO: We should probably use a frame and step through this
        // Also using op_sub is slow. Should write specialized stuff instead.
        for i in 0..len {
          arr.push(
            op_sub(first_param.clone(), Val::Number(i as f64))
              .map_err(|e| e.to_string())
              .unwrap(), // TODO: Exception
          );
        }

        VsArray::from(arr).to_val()
      }
    })
  },
};

static OF: NativeFunction = NativeFunction {
  fn_: |_this: ThisWrapper, params: Vec<Val>| -> Result<Val, Val> {
    Ok(VsArray::from(params).to_val())
  },
};

fn to_array(_: ThisWrapper, params: Vec<Val>) -> Result<Val, Val> {
  if params.len() != 1 {
    return Ok(VsArray::from(params).to_val());
  }

  Ok(match params[0] {
    Val::Number(number) => {
      if number.is_sign_negative() || number != number.floor() {
        return range_error!("Invalid array length");
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
