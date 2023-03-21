use std::rc::Rc;

use crate::format_err;
use crate::native_frame_function::NativeFrameFunction;
use crate::stack_frame::{CallResult, FrameStepOk, FrameStepResult, StackFrameTrait};
use crate::vs_array::VsArray;
use crate::vs_value::{LoadFunctionResult, Val, ValTrait};

pub static REDUCE: NativeFrameFunction = NativeFrameFunction {
  make_frame: || {
    Box::new(ReduceFrame {
      this: None,
      array_i: 0,
      reducer: Val::Void,
      param_i: 0,
      value: None,
    })
  },
};

struct ReduceFrame {
  this: Option<Rc<VsArray>>,
  array_i: usize,

  reducer: Val,
  param_i: usize,
  value: Option<Val>,
}

impl StackFrameTrait for ReduceFrame {
  fn write_this(&mut self, this: Val) {
    self.this = this.as_array_data();
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
      None => return format_err!("TypeError: reduce called on non-array"),
      Some(ad) => ad,
    };

    let array_i = self.array_i;
    self.array_i += 1;

    Ok(match array_data.elements.get(array_i) {
      Some(el) => match el {
        Val::Void => FrameStepOk::Continue,
        _ => match &self.value {
          None => {
            self.value = Some(el.clone());
            FrameStepOk::Continue
          }
          Some(value) => match self.reducer.load_function() {
            LoadFunctionResult::NotAFunction => {
              return format_err!("TypeError: reduce fn is not a function")
            }
            LoadFunctionResult::NativeFunction(native_fn) => {
              self.value = Some(native_fn(
                &mut Val::Undefined,
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
      },
      None => match &self.value {
        None => return format_err!("TypeError: reduce of empty array with no initial value"),
        Some(value) => FrameStepOk::Pop(CallResult {
          return_: value.clone(),
          this: Val::Array(array_data.clone()),
        }),
      },
    })
  }

  fn apply_call_result(&mut self, call_result: CallResult) {
    self.value = Some(call_result.return_);
  }

  fn get_call_result(&mut self) -> CallResult {
    panic!("Not appropriate for ReduceFrame")
  }
}
