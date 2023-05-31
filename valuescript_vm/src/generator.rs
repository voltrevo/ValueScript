use std::{fmt, rc::Rc};

use num_bigint::BigInt;

use crate::{
  builtins::type_error_builtin::ToTypeError,
  iteration::{iteration_result::IterationResult, return_this::RETURN_THIS},
  native_function::{native_fn, NativeFunction},
  stack_frame::StackFrame,
  vs_array::VsArray,
  vs_class::VsClass,
  vs_symbol::VsSymbol,
  vs_value::{ToDynamicVal, ToVal, Val, VsType},
  LoadFunctionResult, ValTrait,
};

#[derive(Clone)]
pub struct Generator {
  #[allow(dead_code)] // TODO
  frame: StackFrame,

  #[allow(dead_code)] // TODO
  stack: Vec<StackFrame>,
}

impl Generator {
  pub fn new(frame: StackFrame) -> Generator {
    return Generator {
      frame,
      stack: vec![],
    };
  }
}

impl ValTrait for Generator {
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

  fn sub(&self, key: Val) -> Result<Val, Val> {
    // TODO: Add symbol for next for performance? (Still needs this fallback)
    if key.to_string() == "next" {
      return Ok(NEXT.to_val());
    }

    if let Val::Symbol(VsSymbol::ITERATOR) = key {
      return Ok(RETURN_THIS.to_val());
    }

    Ok(Val::Undefined)
  }

  fn submov(&mut self, _key: Val, _value: Val) -> Result<(), Val> {
    Err("Cannot assign to subscript of a generator".to_type_error())
  }

  fn pretty_fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "\x1b[36m[Generator]\x1b[39m")
  }

  fn codify(&self) -> String {
    "Generator {{ [native data] }}".to_string()
  }
}

impl fmt::Display for Generator {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "[object Generator]")
  }
}

// We can't use a native function for this. It needs to make a new frame which implements step by
// stepping the contained stack.
//
// Note that next() requires copying the stack in the general case since exceptions must revert the
// iterator. One practical solution for this might be to detect when the generator is in a state
// where it doesn't use the result of the yield expression and reason that a subsequent next() call
// must have the same output, therefore it can store the generated exception in that case instead of
// needing to copy.
//
static NEXT: NativeFunction = native_fn(|mut _this, _| {
  // let dynamic = match this.get_mut()? {
  //   Val::Dynamic(dynamic) => dynamic,
  //   _ => return Err("TODO: indirection".to_error()),
  // };

  // let _generator = dynamic_make_mut(dynamic)
  //   .as_any_mut()
  //   .downcast_mut::<Generator>()
  //   .ok_or_else(|| "Generator.next called on different object".to_type_error())?;

  Ok(
    IterationResult {
      value: "TODO".to_val(),
      done: false,
    }
    .to_dynamic_val(),
  )
});
