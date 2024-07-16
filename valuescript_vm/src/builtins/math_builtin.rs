use std::fmt;
use std::rc::Rc;

use crate::native_function::{native_fn, NativeFunction};
use crate::operations::to_u32;
use crate::vs_class::VsClass;
use crate::vs_value::{LoadFunctionResult, ToVal, Val, ValTrait};

use super::builtin_object::BuiltinObject;

pub struct MathBuiltin {}

impl BuiltinObject for MathBuiltin {
  fn bo_name() -> &'static str {
    "Math"
  }

  fn bo_sub(key: &str) -> Val {
    match key {
      "E" => std::f64::consts::E.to_val(),
      "LN10" => std::f64::consts::LN_10.to_val(),
      "LN2" => std::f64::consts::LN_2.to_val(),
      "LOG10E" => std::f64::consts::LOG10_E.to_val(),
      "LOG2E" => std::f64::consts::LOG2_E.to_val(),
      "PI" => std::f64::consts::PI.to_val(),
      "SQRT1_2" => std::f64::consts::FRAC_1_SQRT_2.to_val(),
      "SQRT2" => std::f64::consts::SQRT_2.to_val(),

      "abs" => ABS.to_val(),
      "acos" => ACOS.to_val(),
      "acosh" => ACOSH.to_val(),
      "asin" => ASIN.to_val(),
      "asinh" => ASINH.to_val(),
      "atan" => ATAN.to_val(),
      "atan2" => ATAN2.to_val(),
      "atanh" => ATANH.to_val(),
      "cbrt" => CBRT.to_val(),
      "ceil" => CEIL.to_val(),
      "clz32" => CLZ32.to_val(),
      "cos" => COS.to_val(),
      "cosh" => COSH.to_val(),
      "exp" => EXP.to_val(),
      "expm1" => EXPM1.to_val(),
      "floor" => FLOOR.to_val(),
      "fround" => FROUND.to_val(),
      "hypot" => HYPOT.to_val(),
      "imul" => IMUL.to_val(),
      "log" => LOG.to_val(),
      "log10" => LOG10.to_val(),
      "log1p" => LOG1P.to_val(),
      "log2" => LOG2.to_val(),
      "max" => MAX.to_val(),
      "min" => MIN.to_val(),
      "pow" => POW.to_val(),
      // random: Not included because it cannot work as expected in ValueScript
      "round" => ROUND.to_val(),
      "sign" => SIGN.to_val(),
      "sin" => SIN.to_val(),
      "sinh" => SINH.to_val(),
      "sqrt" => SQRT.to_val(),
      "tan" => TAN.to_val(),
      "tanh" => TANH.to_val(),
      "trunc" => TRUNC.to_val(),

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

static ABS: NativeFunction = native_fn(|_this, params| {
  let x = param_to_number(params.first());
  Ok(Val::Number(x.abs()))
});

static ACOS: NativeFunction = native_fn(|_this, params| {
  let x = param_to_number(params.first());
  Ok(Val::Number(x.acos()))
});

static ACOSH: NativeFunction = native_fn(|_this, params| {
  let x = param_to_number(params.first());
  Ok(Val::Number(x.acosh()))
});

static ASIN: NativeFunction = native_fn(|_this, params| {
  let x = param_to_number(params.first());
  Ok(Val::Number(x.asin()))
});

static ASINH: NativeFunction = native_fn(|_this, params| {
  let x = param_to_number(params.first());
  Ok(Val::Number(x.sinh()))
});

static ATAN: NativeFunction = native_fn(|_this, params| {
  let x = param_to_number(params.first());
  Ok(Val::Number(x.atan()))
});

static ATAN2: NativeFunction = native_fn(|_this, params| {
  let x = param_to_number(params.first());
  let y = param_to_number(params.get(1));

  Ok(Val::Number(x.atan2(y)))
});

static ATANH: NativeFunction = native_fn(|_this, params| {
  let x = param_to_number(params.first());
  Ok(Val::Number(x.atanh()))
});

static CBRT: NativeFunction = native_fn(|_this, params| {
  let x = param_to_number(params.first());
  Ok(Val::Number(x.cbrt()))
});

static CEIL: NativeFunction = native_fn(|_this, params| {
  let x = param_to_number(params.first());
  Ok(Val::Number(x.ceil()))
});

static CLZ32: NativeFunction = native_fn(|_this, params| {
  let x = param_to_number(params.first());
  Ok(Val::Number(to_u32(x).leading_zeros() as f64))
});

static COS: NativeFunction = native_fn(|_this, params| {
  let x = param_to_number(params.first());
  Ok(Val::Number(x.cos()))
});

static COSH: NativeFunction = native_fn(|_this, params| {
  let x = param_to_number(params.first());
  Ok(Val::Number(x.cosh()))
});

static EXP: NativeFunction = native_fn(|_this, params| {
  let x = param_to_number(params.first());
  Ok(Val::Number(x.exp()))
});

static EXPM1: NativeFunction = native_fn(|_this, params| {
  let x = param_to_number(params.first());
  Ok(Val::Number(x.exp_m1()))
});

static FLOOR: NativeFunction = native_fn(|_this, params| {
  let x = param_to_number(params.first());
  Ok(Val::Number(x.floor()))
});

static FROUND: NativeFunction = native_fn(|_this, params| {
  let x = param_to_number(params.first());
  Ok(Val::Number(x as f32 as f64))
});

static HYPOT: NativeFunction = native_fn(|_this, params| {
  let x = param_to_number(params.first());
  let y = param_to_number(params.get(1));
  Ok(Val::Number(x.hypot(y)))
});

static IMUL: NativeFunction = native_fn(|_this, params| {
  let x = param_to_number(params.first());
  let y = param_to_number(params.get(1));
  Ok(Val::Number((to_u32(x) * to_u32(y)) as i32 as f64))
});

static LOG: NativeFunction = native_fn(|_this, params| {
  let x = param_to_number(params.first());
  Ok(Val::Number(x.ln()))
});

static LOG10: NativeFunction = native_fn(|_this, params| {
  let x = param_to_number(params.first());
  Ok(Val::Number(x.log10()))
});

static LOG1P: NativeFunction = native_fn(|_this, params| {
  let x = param_to_number(params.first());
  Ok(Val::Number(x.ln_1p()))
});

static LOG2: NativeFunction = native_fn(|_this, params| {
  let x = param_to_number(params.first());
  Ok(Val::Number(x.log2()))
});

static MAX: NativeFunction = native_fn(|_this, params| {
  let x = param_to_number(params.first());
  let y = param_to_number(params.get(1));
  Ok(Val::Number(x.max(y)))
});

static MIN: NativeFunction = native_fn(|_this, params| {
  let x = param_to_number(params.first());
  let y = param_to_number(params.get(1));
  Ok(Val::Number(x.min(y)))
});

static POW: NativeFunction = native_fn(|_this, params| {
  let x = param_to_number(params.first());
  let y = param_to_number(params.get(1));
  Ok(Val::Number(x.powf(y)))
});

static ROUND: NativeFunction = native_fn(|_this, params| {
  let x = param_to_number(params.first());
  Ok(Val::Number(x.round()))
});

static SIGN: NativeFunction = native_fn(|_this, params| {
  let x = param_to_number(params.first());
  Ok(Val::Number(x.signum()))
});

static SIN: NativeFunction = native_fn(|_this, params| {
  let x = param_to_number(params.first());
  Ok(Val::Number(x.sin()))
});

static SINH: NativeFunction = native_fn(|_this, params| {
  let x = param_to_number(params.first());
  Ok(Val::Number(x.sinh()))
});

static SQRT: NativeFunction = native_fn(|_this, params| {
  let x = param_to_number(params.first());
  Ok(Val::Number(x.sqrt()))
});

static TAN: NativeFunction = native_fn(|_this, params| {
  let x = param_to_number(params.first());
  Ok(Val::Number(x.tan()))
});

static TANH: NativeFunction = native_fn(|_this, params| {
  let x = param_to_number(params.first());
  Ok(Val::Number(x.tanh()))
});

static TRUNC: NativeFunction = native_fn(|_this, params| {
  let x = param_to_number(params.first());
  Ok(Val::Number(x.trunc()))
});
