use std::rc::Rc;

use crate::native_function::ThisWrapper;
use crate::{builtins::range_error_builtin::to_range_error, range_error};
use crate::{
  format_err, format_val,
  native_function::NativeFunction,
  todo_fn::TODO,
  vs_value::{Val, ValTrait},
};

pub fn op_sub_number(_number: f64, subscript: &Val) -> Val {
  match subscript.val_to_string().as_str() {
    "toExponential" => Val::Static(&TO_EXPONENTIAL),
    "toFixed" => Val::Static(&TO_FIXED),
    "toLocaleString" => Val::Static(&TODO_LOCALE),
    "toPrecision" => Val::Static(&TODO),
    "toString" => Val::Static(&TO_STRING),
    "valueOf" => Val::Static(&VALUE_OF),
    _ => Val::Undefined,
  }
}

static TO_FIXED: NativeFunction = NativeFunction {
  fn_: |this: ThisWrapper, params: Vec<Val>| -> Result<Val, Val> {
    Ok(match this.get() {
      Val::Number(number) => {
        if number.is_infinite() {
          return Ok(if number.is_sign_positive() {
            Val::String(Rc::new("Infinity".to_string()))
          } else {
            Val::String(Rc::new("-Infinity".to_string()))
          });
        }

        let mut precision = match params.get(0) {
          Some(p) => p.to_number(),
          _ => return Ok(Val::String(Rc::new(this.get().val_to_string()))),
        };

        precision = f64::floor(precision);

        if precision < 1.0 || precision > 100.0 {
          return range_error!("precision must be between 1 and 100");
        }

        format_val!("{:.*}", precision as usize, number)
      }
      _ => return format_err!("TODO: number indirection"),
    })
  },
};

static TO_EXPONENTIAL: NativeFunction = NativeFunction {
  fn_: |this: ThisWrapper, params: Vec<Val>| -> Result<Val, Val> {
    Ok(match this.get() {
      Val::Number(number) => match params.get(0) {
        Some(p) => {
          let mut precision = p.to_number();
          precision = f64::floor(precision);

          if precision < 0.0 || precision > 100.0 {
            return range_error!("precision must be between 0 and 100");
          }

          format_exponential(*number, Some(precision as usize))
        }
        None => format_exponential(*number, None),
      },
      _ => return format_err!("number indirection"),
    })
  },
};

static TODO_LOCALE: NativeFunction = NativeFunction {
  fn_: |this: ThisWrapper, _params: Vec<Val>| -> Result<Val, Val> {
    match this.get() {
      Val::Number(_number) => return format_err!("TODO: locale"),
      _ => return format_err!("number indirection"),
    }
  },
};

static TO_STRING: NativeFunction = NativeFunction {
  fn_: |this: ThisWrapper, params: Vec<Val>| -> Result<Val, Val> {
    Ok(match this.get() {
      Val::Number(_) => match params.get(0) {
        Some(_) => {
          return format_err!("TODO: toString with radix");
        }

        None => Val::String(Rc::new(this.get().val_to_string())),
      },
      _ => return format_err!("number indirection"),
    })
  },
};

static VALUE_OF: NativeFunction = NativeFunction {
  fn_: |this: ThisWrapper, _params: Vec<Val>| -> Result<Val, Val> {
    Ok(match this.get() {
      Val::Number(number) => Val::Number(*number),
      _ => return format_err!("number indirection"),
    })
  },
};

fn format_exponential(number: f64, precision: Option<usize>) -> Val {
  if number.is_infinite() {
    return if number.is_sign_positive() {
      Val::String(Rc::new("Infinity".to_string()))
    } else {
      Val::String(Rc::new("-Infinity".to_string()))
    };
  }

  let exp_format = match precision {
    Some(p) => format!("{:.*e}", p, number),
    None => format!("{:e}", number),
  };

  let mut parts = exp_format.splitn(2, 'e');

  let mantissa = parts.next().unwrap();

  let exponent = match parts.next() {
    Some(e) => e,
    None => return Val::String(Rc::new(exp_format)),
  };

  let string = if exponent.starts_with('-') {
    format!("{}e{}", mantissa, exponent)
  } else {
    format!("{}e+{}", mantissa, exponent)
  };

  Val::String(Rc::new(string))
}
