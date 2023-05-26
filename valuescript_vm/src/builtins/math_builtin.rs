use std::fmt;
use std::rc::Rc;

use crate::native_function::{NativeFunction, ThisWrapper};
use crate::operations::to_u32;
use crate::vs_class::VsClass;
use crate::vs_value::{LoadFunctionResult, Val, ValTrait};

use super::builtin_object::BuiltinObject;

pub struct MathBuiltin {}

pub static MATH_BUILTIN: MathBuiltin = MathBuiltin {};

impl BuiltinObject for MathBuiltin {
  fn bo_name() -> &'static str {
    "Math"
  }

  fn bo_sub(key: &str) -> Val {
    match key {
      "E" => Val::Number(std::f64::consts::E),
      "LN10" => Val::Number(std::f64::consts::LN_10),
      "LN2" => Val::Number(std::f64::consts::LN_2),
      "LOG10E" => Val::Number(std::f64::consts::LOG10_E),
      "LOG2E" => Val::Number(std::f64::consts::LOG2_E),
      "PI" => Val::Number(std::f64::consts::PI),
      "SQRT1_2" => Val::Number(std::f64::consts::FRAC_1_SQRT_2),
      "SQRT2" => Val::Number(std::f64::consts::SQRT_2),
      "abs" => Val::Static(&ABS),

      "acos" => Val::Static(&ACOS),
      "acosh" => Val::Static(&ACOSH),
      "asin" => Val::Static(&ASIN),
      "asinh" => Val::Static(&ASINH),
      "atan" => Val::Static(&ATAN),
      "atan2" => Val::Static(&ATAN2),
      "atanh" => Val::Static(&ATANH),
      "cbrt" => Val::Static(&CBRT),
      "ceil" => Val::Static(&CEIL),
      "clz32" => Val::Static(&CLZ32),
      "cos" => Val::Static(&COS),
      "cosh" => Val::Static(&COSH),
      "exp" => Val::Static(&EXP),
      "expm1" => Val::Static(&EXPM1),
      "floor" => Val::Static(&FLOOR),
      "fround" => Val::Static(&FROUND),
      "hypot" => Val::Static(&HYPOT),
      "imul" => Val::Static(&IMUL),
      "log" => Val::Static(&LOG),
      "log10" => Val::Static(&LOG10),
      "log1p" => Val::Static(&LOG1P),
      "log2" => Val::Static(&LOG2),
      "max" => Val::Static(&MAX),
      "min" => Val::Static(&MIN),
      "pow" => Val::Static(&POW),

      // random: Not included because it cannot work as expected in ValueScript
      "round" => Val::Static(&ROUND),
      "sign" => Val::Static(&SIGN),
      "sin" => Val::Static(&SIN),
      "sinh" => Val::Static(&SINH),
      "sqrt" => Val::Static(&SQRT),
      "tan" => Val::Static(&TAN),
      "tanh" => Val::Static(&TANH),
      "trunc" => Val::Static(&TRUNC),

      _ => Val::Undefined,
    }
  }

  fn bo_load_function() -> LoadFunctionResult {
    LoadFunctionResult::NotAFunction
  }

  fn bo_as_class_data() -> Option<Rc<VsClass>> {
    None
  }
}

impl fmt::Display for MathBuiltin {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "[object Math]")
  }
}

fn param_to_number(param: Option<&Val>) -> f64 {
  match param {
    None => f64::NAN,
    Some(p) => p.to_number(),
  }
}

static ABS: NativeFunction = NativeFunction {
  fn_: |_this: ThisWrapper, params: Vec<Val>| -> Result<Val, Val> {
    let x = param_to_number(params.get(0));
    Ok(Val::Number(x.abs()))
  },
};

static ACOS: NativeFunction = NativeFunction {
  fn_: |_this: ThisWrapper, params: Vec<Val>| -> Result<Val, Val> {
    let x = param_to_number(params.get(0));
    Ok(Val::Number(x.acos()))
  },
};

static ACOSH: NativeFunction = NativeFunction {
  fn_: |_this: ThisWrapper, params: Vec<Val>| -> Result<Val, Val> {
    let x = param_to_number(params.get(0));
    Ok(Val::Number(x.acosh()))
  },
};

static ASIN: NativeFunction = NativeFunction {
  fn_: |_this: ThisWrapper, params: Vec<Val>| -> Result<Val, Val> {
    let x = param_to_number(params.get(0));
    Ok(Val::Number(x.asin()))
  },
};

static ASINH: NativeFunction = NativeFunction {
  fn_: |_this: ThisWrapper, params: Vec<Val>| -> Result<Val, Val> {
    let x = param_to_number(params.get(0));
    Ok(Val::Number(x.sinh()))
  },
};

static ATAN: NativeFunction = NativeFunction {
  fn_: |_this: ThisWrapper, params: Vec<Val>| -> Result<Val, Val> {
    let x = param_to_number(params.get(0));
    Ok(Val::Number(x.atan()))
  },
};

static ATAN2: NativeFunction = NativeFunction {
  fn_: |_this: ThisWrapper, params: Vec<Val>| -> Result<Val, Val> {
    let x = param_to_number(params.get(0));
    let y = param_to_number(params.get(1));

    Ok(Val::Number(x.atan2(y)))
  },
};

static ATANH: NativeFunction = NativeFunction {
  fn_: |_this: ThisWrapper, params: Vec<Val>| -> Result<Val, Val> {
    let x = param_to_number(params.get(0));
    Ok(Val::Number(x.atanh()))
  },
};

static CBRT: NativeFunction = NativeFunction {
  fn_: |_this: ThisWrapper, params: Vec<Val>| -> Result<Val, Val> {
    let x = param_to_number(params.get(0));
    Ok(Val::Number(x.cbrt()))
  },
};

static CEIL: NativeFunction = NativeFunction {
  fn_: |_this: ThisWrapper, params: Vec<Val>| -> Result<Val, Val> {
    let x = param_to_number(params.get(0));
    Ok(Val::Number(x.ceil()))
  },
};

static CLZ32: NativeFunction = NativeFunction {
  fn_: |_this: ThisWrapper, params: Vec<Val>| -> Result<Val, Val> {
    let x = param_to_number(params.get(0));
    Ok(Val::Number(to_u32(x).leading_zeros() as f64))
  },
};

static COS: NativeFunction = NativeFunction {
  fn_: |_this: ThisWrapper, params: Vec<Val>| -> Result<Val, Val> {
    let x = param_to_number(params.get(0));
    Ok(Val::Number(x.cos()))
  },
};

static COSH: NativeFunction = NativeFunction {
  fn_: |_this: ThisWrapper, params: Vec<Val>| -> Result<Val, Val> {
    let x = param_to_number(params.get(0));
    Ok(Val::Number(x.cosh()))
  },
};

static EXP: NativeFunction = NativeFunction {
  fn_: |_this: ThisWrapper, params: Vec<Val>| -> Result<Val, Val> {
    let x = param_to_number(params.get(0));
    Ok(Val::Number(x.exp()))
  },
};

static EXPM1: NativeFunction = NativeFunction {
  fn_: |_this: ThisWrapper, params: Vec<Val>| -> Result<Val, Val> {
    let x = param_to_number(params.get(0));
    Ok(Val::Number(x.exp_m1()))
  },
};

static FLOOR: NativeFunction = NativeFunction {
  fn_: |_this: ThisWrapper, params: Vec<Val>| -> Result<Val, Val> {
    let x = param_to_number(params.get(0));
    Ok(Val::Number(x.floor()))
  },
};

static FROUND: NativeFunction = NativeFunction {
  fn_: |_this: ThisWrapper, params: Vec<Val>| -> Result<Val, Val> {
    let x = param_to_number(params.get(0));
    Ok(Val::Number(x as f32 as f64))
  },
};

static HYPOT: NativeFunction = NativeFunction {
  fn_: |_this: ThisWrapper, params: Vec<Val>| -> Result<Val, Val> {
    let x = param_to_number(params.get(0));
    let y = param_to_number(params.get(1));
    Ok(Val::Number(x.hypot(y)))
  },
};

static IMUL: NativeFunction = NativeFunction {
  fn_: |_this: ThisWrapper, params: Vec<Val>| -> Result<Val, Val> {
    let x = param_to_number(params.get(0));
    let y = param_to_number(params.get(1));
    Ok(Val::Number((to_u32(x) * to_u32(y)) as i32 as f64))
  },
};

static LOG: NativeFunction = NativeFunction {
  fn_: |_this: ThisWrapper, params: Vec<Val>| -> Result<Val, Val> {
    let x = param_to_number(params.get(0));
    Ok(Val::Number(x.ln()))
  },
};

static LOG10: NativeFunction = NativeFunction {
  fn_: |_this: ThisWrapper, params: Vec<Val>| -> Result<Val, Val> {
    let x = param_to_number(params.get(0));
    Ok(Val::Number(x.log10()))
  },
};

static LOG1P: NativeFunction = NativeFunction {
  fn_: |_this: ThisWrapper, params: Vec<Val>| -> Result<Val, Val> {
    let x = param_to_number(params.get(0));
    Ok(Val::Number(x.ln_1p()))
  },
};

static LOG2: NativeFunction = NativeFunction {
  fn_: |_this: ThisWrapper, params: Vec<Val>| -> Result<Val, Val> {
    let x = param_to_number(params.get(0));
    Ok(Val::Number(x.log2()))
  },
};

static MAX: NativeFunction = NativeFunction {
  fn_: |_this: ThisWrapper, params: Vec<Val>| -> Result<Val, Val> {
    let x = param_to_number(params.get(0));
    let y = param_to_number(params.get(1));
    Ok(Val::Number(x.max(y)))
  },
};

static MIN: NativeFunction = NativeFunction {
  fn_: |_this: ThisWrapper, params: Vec<Val>| -> Result<Val, Val> {
    let x = param_to_number(params.get(0));
    let y = param_to_number(params.get(1));
    Ok(Val::Number(x.min(y)))
  },
};

static POW: NativeFunction = NativeFunction {
  fn_: |_this: ThisWrapper, params: Vec<Val>| -> Result<Val, Val> {
    let x = param_to_number(params.get(0));
    let y = param_to_number(params.get(1));
    Ok(Val::Number(x.powf(y)))
  },
};

static ROUND: NativeFunction = NativeFunction {
  fn_: |_this: ThisWrapper, params: Vec<Val>| -> Result<Val, Val> {
    let x = param_to_number(params.get(0));
    Ok(Val::Number(x.round()))
  },
};

static SIGN: NativeFunction = NativeFunction {
  fn_: |_this: ThisWrapper, params: Vec<Val>| -> Result<Val, Val> {
    let x = param_to_number(params.get(0));
    Ok(Val::Number(x.signum()))
  },
};

static SIN: NativeFunction = NativeFunction {
  fn_: |_this: ThisWrapper, params: Vec<Val>| -> Result<Val, Val> {
    let x = param_to_number(params.get(0));
    Ok(Val::Number(x.sin()))
  },
};

static SINH: NativeFunction = NativeFunction {
  fn_: |_this: ThisWrapper, params: Vec<Val>| -> Result<Val, Val> {
    let x = param_to_number(params.get(0));
    Ok(Val::Number(x.sinh()))
  },
};

static SQRT: NativeFunction = NativeFunction {
  fn_: |_this: ThisWrapper, params: Vec<Val>| -> Result<Val, Val> {
    let x = param_to_number(params.get(0));
    Ok(Val::Number(x.sqrt()))
  },
};

static TAN: NativeFunction = NativeFunction {
  fn_: |_this: ThisWrapper, params: Vec<Val>| -> Result<Val, Val> {
    let x = param_to_number(params.get(0));
    Ok(Val::Number(x.tan()))
  },
};

static TANH: NativeFunction = NativeFunction {
  fn_: |_this: ThisWrapper, params: Vec<Val>| -> Result<Val, Val> {
    let x = param_to_number(params.get(0));
    Ok(Val::Number(x.tanh()))
  },
};

static TRUNC: NativeFunction = NativeFunction {
  fn_: |_this: ThisWrapper, params: Vec<Val>| -> Result<Val, Val> {
    let x = param_to_number(params.get(0));
    Ok(Val::Number(x.trunc()))
  },
};
