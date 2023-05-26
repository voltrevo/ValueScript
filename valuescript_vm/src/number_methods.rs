use crate::builtins::error_builtin::ToError;
use crate::native_function::ThisWrapper;
use crate::vs_value::ToVal;
use crate::{builtins::range_error_builtin::to_range_error, range_error};
use crate::{
  native_function::NativeFunction,
  todo_fn::TODO,
  vs_value::{Val, ValTrait},
};

pub fn op_sub_number(_number: f64, subscript: &Val) -> Val {
  match subscript.to_string().as_str() {
    "toExponential" => &TO_EXPONENTIAL,
    "toFixed" => &TO_FIXED,
    "toLocaleString" => &TODO_LOCALE,
    "toPrecision" => &TODO,
    "toString" => &TO_STRING,
    "valueOf" => &VALUE_OF,
    _ => return Val::Undefined,
  }
  .to_val()
}

static TO_FIXED: NativeFunction = NativeFunction {
  fn_: |this: ThisWrapper, params: Vec<Val>| -> Result<Val, Val> {
    Ok(match this.get() {
      Val::Number(number) => {
        if number.is_infinite() {
          return Ok(
            if number.is_sign_positive() {
              "Infinity"
            } else {
              "-Infinity"
            }
            .to_val(),
          );
        }

        let mut precision = match params.get(0) {
          Some(p) => p.to_number(),
          _ => return Ok(number.to_val().to_string().to_val()),
        };

        precision = f64::floor(precision);

        if precision < 1.0 || precision > 100.0 {
          return range_error!("precision must be between 1 and 100");
        }

        format!("{:.*}", precision as usize, number).to_val()
      }
      _ => return Err(format!("TODO: number indirection").to_val()),
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
      _ => return Err("number indirection".to_error()),
    })
  },
};

static TODO_LOCALE: NativeFunction = NativeFunction {
  fn_: |this: ThisWrapper, _params: Vec<Val>| -> Result<Val, Val> {
    match this.get() {
      Val::Number(_number) => return Err("TODO: locale".to_error()),
      _ => return Err("number indirection".to_error()),
    }
  },
};

static TO_STRING: NativeFunction = NativeFunction {
  fn_: |this: ThisWrapper, params: Vec<Val>| -> Result<Val, Val> {
    Ok(match this.get() {
      Val::Number(number) => match params.get(0) {
        Some(_) => {
          return Err("TODO: toString with radix".to_error());
        }

        None => number.to_val().to_string().to_val(),
      },
      _ => return Err("number indirection".to_error()),
    })
  },
};

static VALUE_OF: NativeFunction = NativeFunction {
  fn_: |this: ThisWrapper, _params: Vec<Val>| -> Result<Val, Val> {
    Ok(match this.get() {
      Val::Number(number) => Val::Number(*number),
      _ => return Err("number indirection".to_error()),
    })
  },
};

fn format_exponential(number: f64, precision: Option<usize>) -> Val {
  if number.is_infinite() {
    return if number.is_sign_positive() {
      "Infinity"
    } else {
      "-Infinity"
    }
    .to_val();
  }

  let exp_format = match precision {
    Some(p) => format!("{:.*e}", p, number),
    None => format!("{:e}", number),
  };

  let mut parts = exp_format.splitn(2, 'e');

  let mantissa = parts.next().unwrap();

  let exponent = match parts.next() {
    Some(e) => e,
    None => return exp_format.to_val(),
  };

  let string = if exponent.starts_with('-') {
    format!("{}e{}", mantissa, exponent)
  } else {
    format!("{}e+{}", mantissa, exponent)
  };

  string.to_val()
}
