use std::{fmt, mem::take, rc::Rc};

use num_bigint::BigInt;

use crate::{
  builtins::{error_builtin::ToError, type_error_builtin::ToTypeError},
  iteration::{iteration_result::IterationResult, return_this::RETURN_THIS},
  native_frame_function::NativeFrameFunction,
  stack_frame::{CallResult, FrameStepOk, FrameStepResult, StackFrame, StackFrameTrait},
  vs_array::VsArray,
  vs_class::VsClass,
  vs_symbol::VsSymbol,
  vs_value::{dynamic_make_mut, ToDynamicVal, ToVal, Val, VsType},
  LoadFunctionResult, ValTrait,
};

#[derive(Clone, Default)]
pub struct Generator {
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

// Note that next() requires copying the stack in the general case since exceptions must revert the
// iterator. One practical solution for this might be to detect when the generator is in a state
// where it doesn't use the result of the yield expression and reason that a subsequent next() call
// must have the same output, therefore it can store the generated exception in that case instead of
// needing to copy.
//
static NEXT: NativeFrameFunction = NativeFrameFunction {
  make_frame: || Box::new(GeneratorFrame::default()),
};

#[derive(Clone, Default)]
struct GeneratorFrame {
  generator: Generator,
}

impl GeneratorFrame {}

impl StackFrameTrait for GeneratorFrame {
  fn write_this(&mut self, const_: bool, this: Val) -> Result<(), Val> {
    let mut dynamic = match this {
      Val::Dynamic(dynamic) => dynamic,
      _ => return Err("TODO: indirection".to_error()),
    };

    if const_ {
      return Err("Cannot call Generator.next on a const generator".to_type_error());
    }

    let mut generator = dynamic_make_mut(&mut dynamic)
      .as_any_mut()
      .downcast_mut::<Generator>()
      .ok_or_else(|| "Generator.next called on different object".to_type_error())?;

    self.generator = take(&mut generator);

    Ok(())
  }

  fn write_param(&mut self, _param: Val) {
    panic!("TODO: results of yield expressions")
  }

  fn step(&mut self) -> FrameStepResult {
    let fsr = self.generator.frame.step();

    match fsr {
      Err(_) => fsr, // TODO: Stack unwind internal stack first
      Ok(FrameStepOk::Continue) | Ok(FrameStepOk::Push(_)) => fsr,
      Ok(FrameStepOk::Pop(call_result)) => Ok(FrameStepOk::Pop(CallResult {
        return_: IterationResult {
          value: call_result.return_, // TODO: Assert call_result.this is undefined?
          done: true,
        }
        .to_dynamic_val(),
        this: take(&mut self.generator).to_dynamic_val(),
      })),
      Ok(FrameStepOk::Yield(val)) => Ok(FrameStepOk::Pop(CallResult {
        return_: IterationResult {
          value: val,
          done: false,
        }
        .to_dynamic_val(),
        this: take(&mut self.generator).to_dynamic_val(),
      })),
    }
  }

  fn apply_call_result(&mut self, call_result: CallResult) {
    self.generator.frame.apply_call_result(call_result)
  }

  fn get_call_result(&mut self) -> CallResult {
    panic!("Not appropriate for GeneratorFrame")
  }

  fn catch_exception(&mut self, exception: Val) -> bool {
    self.generator.frame.catch_exception(exception)
  }

  fn clone_to_stack_frame(&self) -> StackFrame {
    Box::new(self.clone())
  }
}
