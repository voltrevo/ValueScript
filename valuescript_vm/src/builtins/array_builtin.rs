use std::rc::Rc;

use num_bigint::BigInt;

use crate::{
  builtins::range_error_builtin::to_range_error,
  builtins::type_error_builtin::to_type_error,
  format_err,
  native_function::NativeFunction,
  operations::op_sub,
  range_error, type_error,
  vs_array::VsArray,
  vs_class::VsClass,
  vs_object::VsObject,
  vs_value::{LoadFunctionResult, Val, VsType},
  ValTrait,
};

pub struct ArrayBuiltin {}

pub static ARRAY_BUILTIN: ArrayBuiltin = ArrayBuiltin {};

impl ValTrait for ArrayBuiltin {
  fn typeof_(&self) -> VsType {
    VsType::Object
  }
  fn val_to_string(&self) -> String {
    "function Array() { [native code] }".to_string()
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
    Val::String(Rc::new("function Array() { [native code] }".to_string()))
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
    Ok(Val::Static(match key.val_to_string().as_str() {
      "isArray" => &IS_ARRAY,
      "from" => &FROM,
      "of" => &OF,
      _ => return Ok(Val::Undefined),
    }))
  }

  fn submov(&mut self, _key: Val, _value: Val) -> Result<(), Val> {
    type_error!("Cannot assign to subscript of Array builtin")
  }

  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "\x1b[36m[Array]\x1b[39m")
  }

  fn codify(&self) -> String {
    "Array".into()
  }
}

static IS_ARRAY: NativeFunction = NativeFunction {
  fn_: |_this: &mut Val, params: Vec<Val>| -> Result<Val, Val> {
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
  fn_: |_this: &mut Val, params: Vec<Val>| -> Result<Val, Val> {
    let first_param = match params.get(0) {
      None => return type_error!("undefined is not iterable"),
      Some(p) => p,
    };

    if params.len() > 1 {
      return format_err!("TODO: Using Array.from with a map function");
    }

    Ok(match first_param {
      Val::Array(arr) => Val::Array(arr.clone()),
      Val::String(s) => Val::Array(Rc::new(VsArray::from(
        s.chars()
          .map(|c| Val::String(Rc::new(c.to_string())))
          .collect(),
      ))),
      Val::Void | Val::Undefined | Val::Null => return type_error!("items is not iterable"),
      Val::Bool(..) | Val::Number(..) | Val::BigInt(..) => Val::Array(Rc::new(VsArray::new())),
      Val::Object(..) | Val::Function(..) | Val::Class(..) | Val::Static(..) | Val::Custom(..) => {
        let len = op_sub(
          first_param.clone(),
          Val::String(Rc::new("length".to_string())),
        )
        .map_err(|e| e.val_to_string())
        .unwrap() // TODO: Exception
        .to_number();

        if len.is_sign_negative() || len.is_nan() {
          return Ok(Val::Array(Rc::new(VsArray::new())));
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
              .map_err(|e| e.val_to_string())
              .unwrap(), // TODO: Exception
          );
        }

        Val::Array(Rc::new(VsArray::from(arr)))
      }
    })
  },
};

static OF: NativeFunction = NativeFunction {
  fn_: |_this: &mut Val, params: Vec<Val>| -> Result<Val, Val> {
    Ok(Val::Array(Rc::new(VsArray::from(params))))
  },
};

fn to_array(_: &mut Val, params: Vec<Val>) -> Result<Val, Val> {
  if params.len() != 1 {
    return Ok(Val::Array(Rc::new(VsArray::from(params))));
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

      Val::Array(Rc::new(VsArray::from(arr)))
    }
    _ => Val::Array(Rc::new(VsArray::from(params))),
  })
}
