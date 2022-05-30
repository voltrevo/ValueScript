use std::rc::Rc;

use super::super::vs_value::{Val, ValTrait, LoadFunctionResult};
use super::super::vs_array::VsArray;
use super::super::native_frame_function::NativeFrameFunction;
use super::super::stack_frame::{StackFrameTrait, FrameStepResult, CallResult};

pub static EVERY: NativeFrameFunction = NativeFrameFunction {
  make_frame: || Box::new(EveryFrame {
    this: None,
    this_arg: Val::Undefined,
    condition_fn: Val::Void,
    param_i: 0,
    array_i: 0,
    falsified: false,
  }),
};

struct EveryFrame {
  this: Option<Rc<VsArray>>,
  this_arg: Val,
  condition_fn: Val,
  param_i: usize,
  array_i: usize,
  falsified: bool,
}

impl StackFrameTrait for EveryFrame {
  fn write_this(&mut self, this: Val) {
    self.this = this.as_array_data();
  }

  fn write_param(&mut self, param: Val) {
    match self.param_i {
      0 => { self.condition_fn = param; }
      1 => { self.this_arg = param; }
      _ => {},
    };

    self.param_i += 1;
  }

  fn step(&mut self) -> FrameStepResult {
    let array_data = match &self.this {
      None => std::panic!("Not implemented: exception: every called on non-array"),
      Some(ad) => ad,
    };

    if self.falsified {
      return FrameStepResult::Pop(CallResult {
        return_: Val::Bool(false),
        this: Val::Array(array_data.clone()),
      });
    }

    let index = Val::Number(self.array_i as f64);
    self.array_i += 1;

    match array_data.elements.get(self.array_i) {
      Some(el) => match el {
        Val::Void => {
          return FrameStepResult::Continue;
        },
        _ => match self.condition_fn.load_function() {
          LoadFunctionResult::NotAFunction =>
            std::panic!("Not implemented: exception: map fn is not a function")
          ,
          LoadFunctionResult::NativeFunction(native_fn) => {
            let cond_result = native_fn(
              &mut self.this_arg.clone(),
              vec![
                el.clone(),
                index,
                Val::Array(array_data.clone()),
              ],
            );

            if !cond_result.is_truthy() {
              return FrameStepResult::Pop(CallResult {
                return_: Val::Bool(false),
                this: Val::Array(array_data.clone()),
              });
            }
  
            return FrameStepResult::Continue;
          },
          LoadFunctionResult::StackFrame(mut new_frame) => {
            new_frame.write_this(self.this_arg.clone());
            new_frame.write_param(el.clone());
            new_frame.write_param(index);
            new_frame.write_param(Val::Array(array_data.clone()));
            return FrameStepResult::Push(new_frame);
          },
        },
      },
      None => {
        return FrameStepResult::Pop(CallResult {
          return_: Val::Bool(true),
          this: Val::Array(array_data.clone()),
        });
      },
    };
  }

  fn apply_call_result(&mut self, call_result: CallResult) {
    if !call_result.return_.is_truthy() {
      self.falsified = true;
    }
  }

  fn get_call_result(&mut self) -> CallResult {
    std::panic!("Not appropriate for EveryFrame")
  }
}
