use std::rc::Rc;

use crate::builtins::type_error_builtin::ToTypeError;
use crate::native_frame_function::NativeFrameFunction;
use crate::native_function::ThisWrapper;
use crate::stack_frame::{CallResult, FrameStepOk, FrameStepResult, StackFrameTrait};
use crate::vs_array::VsArray;
use crate::vs_value::{LoadFunctionResult, Val, ValTrait};

pub static REDUCE_RIGHT: NativeFrameFunction = NativeFrameFunction {
  make_frame: || {
    Box::new(ReduceRightFrame {
      this: None,
      array_i: 0,
      reducer: Val::Void,
      param_i: 0,
      value: None,
    })
  },
};

struct ReduceRightFrame {
  this: Option<Rc<VsArray>>,
  array_i: usize,

  reducer: Val,
  param_i: usize,
  value: Option<Val>,
}

impl StackFrameTrait for ReduceRightFrame {
  fn write_this(&mut self, _const: bool, this: Val) -> Result<(), Val> {
    self.this = this.as_array_data();

    match &self.this {
      None => {}
      Some(ad) => self.array_i = ad.elements.len(),
    };

    Ok(())
  }

  fn write_param(&mut self, param: Val) {
    match self.param_i {
      0 => {
        self.reducer = param;
      }
      1 => {
        self.value = Some(param);
      }
      _ => {}
    };

    self.param_i += 1;
  }

  fn step(&mut self) -> FrameStepResult {
    let array_data = match &self.this {
      None => return Err("reduceRight called on non-array".to_type_error()),
      Some(ad) => ad,
    };

    if self.array_i == 0 {
      match &self.value {
        None => {
          return Err("reduceRight of empty array with no initial value".to_type_error());
        }
        Some(value) => {
          return Ok(FrameStepOk::Pop(CallResult {
            return_: value.clone(),
            this: Val::Array(array_data.clone()),
          }));
        }
      }
    }

    self.array_i -= 1;
    let array_i = self.array_i;

    let el = &array_data.elements[array_i];

    Ok(match el {
      Val::Void => FrameStepOk::Continue,
      _ => match &self.value {
        None => {
          self.value = Some(el.clone());
          FrameStepOk::Continue
        }
        Some(value) => match self.reducer.load_function() {
          LoadFunctionResult::NotAFunction => {
            return Err("reduceRight fn is not a function".to_type_error())
          }
          LoadFunctionResult::NativeFunction(native_fn) => {
            self.value = Some(native_fn(
              ThisWrapper::new(true, &mut Val::Undefined),
              vec![
                value.clone(),
                el.clone(),
                Val::Number(array_i as f64),
                Val::Array(array_data.clone()),
              ],
            )?);

            FrameStepOk::Continue
          }
          LoadFunctionResult::StackFrame(mut new_frame) => {
            new_frame.write_param(value.clone());
            new_frame.write_param(el.clone());
            new_frame.write_param(Val::Number(array_i as f64));
            new_frame.write_param(Val::Array(array_data.clone()));
            FrameStepOk::Push(new_frame)
          }
        },
      },
    })
  }

  fn apply_call_result(&mut self, call_result: CallResult) {
    self.value = Some(call_result.return_);
  }

  fn get_call_result(&mut self) -> CallResult {
    panic!("Not appropriate for ReduceRightFrame")
  }

  fn catch_exception(&mut self, _exception: Val) -> bool {
    return false;
  }
}
