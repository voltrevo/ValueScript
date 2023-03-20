use std::rc::Rc;

use crate::{
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
  fn_: |this: &mut Val, params: Vec<Val>| -> Val {
    match this {
      Val::Number(number) => {
        if number.is_infinite() {
          return if number.is_sign_positive() {
            Val::String(Rc::new("Infinity".to_string()))
          } else {
            Val::String(Rc::new("-Infinity".to_string()))
          };
        }

        let mut precision = match params.get(0) {
          Some(p) => p.to_number(),
          _ => return Val::String(Rc::new(this.val_to_string())),
        };

        precision = f64::floor(precision);

        if precision < 1.0 || precision > 100.0 {
          panic!("TODO: exceptions: RangeError: precision must be between 1 and 100");
        }

        Val::String(Rc::new(format!("{:.*}", precision as usize, number)))
      }
      _ => panic!("TODO: exceptions/number indirection"),
    }
  },
};

static TO_EXPONENTIAL: NativeFunction = NativeFunction {
  fn_: |this: &mut Val, params: Vec<Val>| -> Val {
    match this {
      Val::Number(number) => match params.get(0) {
        Some(p) => {
          let mut precision = p.to_number();
          precision = f64::floor(precision);

          if precision < 0.0 || precision > 100.0 {
            panic!("TODO: exceptions: RangeError: precision must be between 0 and 100");
          }

          format_exponential(*number, Some(precision as usize))
        }
        None => format_exponential(*number, None),
      },
      _ => panic!("TODO: exceptions/number indirection"),
    }
  },
};

static TODO_LOCALE: NativeFunction = NativeFunction {
  fn_: |this: &mut Val, _params: Vec<Val>| -> Val {
    match this {
      Val::Number(_number) => panic!("TODO: locale"),
      _ => panic!("TODO: exceptions/number indirection"),
    }
  },
};

static TO_STRING: NativeFunction = NativeFunction {
  fn_: |this: &mut Val, params: Vec<Val>| -> Val {
    match this {
      Val::Number(_) => match params.get(0) {
        Some(_) => {
          panic!("TODO: toString with radix");
        }

        None => Val::String(Rc::new(this.val_to_string())),
      },
      _ => panic!("TODO: exceptions/number indirection"),
    }
  },
};

static VALUE_OF: NativeFunction = NativeFunction {
  fn_: |this: &mut Val, _params: Vec<Val>| -> Val {
    match this {
      Val::Number(number) => Val::Number(*number),
      _ => panic!("TODO: exceptions/number indirection"),
    }
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
