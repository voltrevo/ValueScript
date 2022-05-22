use std::rc::Rc;

use super::vs_value::{Val, VsType, ValTrait, LoadFunctionResult};
use super::vs_array::VsArray;
use super::vs_object::VsObject;
use super::native_function::NativeFunction;

pub struct Math {}

pub static MATH: Math = Math {};

impl ValTrait for Math {
  fn typeof_(&self) -> VsType { VsType::Object }
  fn val_to_string(&self) -> String { "[object Math]".to_string() }
  fn to_number(&self) -> f64 { f64::NAN }
  fn to_index(&self) -> Option<usize> { None }
  fn is_primitive(&self) -> bool { false }
  fn to_primitive(&self) -> Val { Val::String(Rc::new(self.val_to_string())) }
  fn is_truthy(&self) -> bool { true }
  fn is_nullish(&self) -> bool { false }

  fn bind(&self, _params: Vec<Val>) -> Option<Val> { None }

  fn as_array_data(&self) -> Option<Rc<VsArray>> { None }
  fn as_object_data(&self) -> Option<Rc<VsObject>> { None }

  fn load_function(&self) -> LoadFunctionResult {
    LoadFunctionResult::NotAFunction
  }

  fn sub(&self, key: Val) -> Val {
    match key.val_to_string().as_str() {
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

  fn submov(&mut self, _key: Val, _value: Val) {
    std::panic!("Not implemented: exceptions");
  }

  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "\x1b[36m[Math]\x1b[39m")
  }
}

fn param_to_number(param: Option<&Val>) -> f64 {
  match param {
    None => f64::NAN,
    Some(p) => p.to_number(),
  }
}

static ABS: NativeFunction = NativeFunction {
  fn_: |_this: &mut Val, params: Vec<Val>| -> Val {
    let x = param_to_number(params.get(0));

    return Val::Number(x.abs());
  }
};

static ACOS: NativeFunction = NativeFunction {
  fn_: |_this: &mut Val, _params: Vec<Val>| -> Val {
    std::panic!("Not implemented: ACOS");
  }
};

static ACOSH: NativeFunction = NativeFunction {
  fn_: |_this: &mut Val, _params: Vec<Val>| -> Val {
    std::panic!("Not implemented: ACOSH");
  }
};

static ASIN: NativeFunction = NativeFunction {
  fn_: |_this: &mut Val, _params: Vec<Val>| -> Val {
    std::panic!("Not implemented: ASIN");
  }
};

static ASINH: NativeFunction = NativeFunction {
  fn_: |_this: &mut Val, _params: Vec<Val>| -> Val {
    std::panic!("Not implemented: ASINH");
  }
};

static ATAN: NativeFunction = NativeFunction {
  fn_: |_this: &mut Val, _params: Vec<Val>| -> Val {
    std::panic!("Not implemented: ATAN");
  }
};

static ATAN2: NativeFunction = NativeFunction {
  fn_: |_this: &mut Val, _params: Vec<Val>| -> Val {
    std::panic!("Not implemented: ATAN2");
  }
};

static ATANH: NativeFunction = NativeFunction {
  fn_: |_this: &mut Val, _params: Vec<Val>| -> Val {
    std::panic!("Not implemented: ATANH");
  }
};

static CBRT: NativeFunction = NativeFunction {
  fn_: |_this: &mut Val, _params: Vec<Val>| -> Val {
    std::panic!("Not implemented: CBRT");
  }
};

static CEIL: NativeFunction = NativeFunction {
  fn_: |_this: &mut Val, _params: Vec<Val>| -> Val {
    std::panic!("Not implemented: CEIL");
  }
};

static CLZ32: NativeFunction = NativeFunction {
  fn_: |_this: &mut Val, _params: Vec<Val>| -> Val {
    std::panic!("Not implemented: CLZ32");
  }
};

static COS: NativeFunction = NativeFunction {
  fn_: |_this: &mut Val, _params: Vec<Val>| -> Val {
    std::panic!("Not implemented: COS");
  }
};

static COSH: NativeFunction = NativeFunction {
  fn_: |_this: &mut Val, _params: Vec<Val>| -> Val {
    std::panic!("Not implemented: COSH");
  }
};

static EXP: NativeFunction = NativeFunction {
  fn_: |_this: &mut Val, _params: Vec<Val>| -> Val {
    std::panic!("Not implemented: EXP");
  }
};

static EXPM1: NativeFunction = NativeFunction {
  fn_: |_this: &mut Val, _params: Vec<Val>| -> Val {
    std::panic!("Not implemented: EXPM1");
  }
};

static FLOOR: NativeFunction = NativeFunction {
  fn_: |_this: &mut Val, _params: Vec<Val>| -> Val {
    std::panic!("Not implemented: FLOOR");
  }
};

static FROUND: NativeFunction = NativeFunction {
  fn_: |_this: &mut Val, _params: Vec<Val>| -> Val {
    std::panic!("Not implemented: FROUND");
  }
};

static HYPOT: NativeFunction = NativeFunction {
  fn_: |_this: &mut Val, _params: Vec<Val>| -> Val {
    std::panic!("Not implemented: HYPOT");
  }
};

static IMUL: NativeFunction = NativeFunction {
  fn_: |_this: &mut Val, _params: Vec<Val>| -> Val {
    std::panic!("Not implemented: IMUL");
  }
};

static LOG: NativeFunction = NativeFunction {
  fn_: |_this: &mut Val, _params: Vec<Val>| -> Val {
    std::panic!("Not implemented: LOG");
  }
};

static LOG10: NativeFunction = NativeFunction {
  fn_: |_this: &mut Val, _params: Vec<Val>| -> Val {
    std::panic!("Not implemented: LOG10");
  }
};

static LOG1P: NativeFunction = NativeFunction {
  fn_: |_this: &mut Val, _params: Vec<Val>| -> Val {
    std::panic!("Not implemented: LOG1P");
  }
};

static LOG2: NativeFunction = NativeFunction {
  fn_: |_this: &mut Val, _params: Vec<Val>| -> Val {
    std::panic!("Not implemented: LOG2");
  }
};

static MAX: NativeFunction = NativeFunction {
  fn_: |_this: &mut Val, _params: Vec<Val>| -> Val {
    std::panic!("Not implemented: MAX");
  }
};

static MIN: NativeFunction = NativeFunction {
  fn_: |_this: &mut Val, _params: Vec<Val>| -> Val {
    std::panic!("Not implemented: MIN");
  }
};

static POW: NativeFunction = NativeFunction {
  fn_: |_this: &mut Val, _params: Vec<Val>| -> Val {
    std::panic!("Not implemented: POW");
  }
};

static ROUND: NativeFunction = NativeFunction {
  fn_: |_this: &mut Val, _params: Vec<Val>| -> Val {
    std::panic!("Not implemented: ROUND");
  }
};

static SIGN: NativeFunction = NativeFunction {
  fn_: |_this: &mut Val, _params: Vec<Val>| -> Val {
    std::panic!("Not implemented: SIGN");
  }
};

static SIN: NativeFunction = NativeFunction {
  fn_: |_this: &mut Val, _params: Vec<Val>| -> Val {
    std::panic!("Not implemented: SIN");
  }
};

static SINH: NativeFunction = NativeFunction {
  fn_: |_this: &mut Val, _params: Vec<Val>| -> Val {
    std::panic!("Not implemented: SINH");
  }
};

static SQRT: NativeFunction = NativeFunction {
  fn_: |_this: &mut Val, _params: Vec<Val>| -> Val {
    std::panic!("Not implemented: SQRT");
  }
};

static TAN: NativeFunction = NativeFunction {
  fn_: |_this: &mut Val, _params: Vec<Val>| -> Val {
    std::panic!("Not implemented: TAN");
  }
};

static TANH: NativeFunction = NativeFunction {
  fn_: |_this: &mut Val, _params: Vec<Val>| -> Val {
    std::panic!("Not implemented: TANH");
  }
};

static TRUNC: NativeFunction = NativeFunction {
  fn_: |_this: &mut Val, _params: Vec<Val>| -> Val {
    std::panic!("Not implemented: TRUNC");
  }
};

