use std::rc::Rc;

use super::super::vs_value::{Val, ValTrait, LoadFunctionResult};
use super::super::vs_array::VsArray;
use super::super::native_frame_function::NativeFrameFunction;
use super::super::stack_frame::{StackFrameTrait, FrameStepResult, CallResult};

pub static FIND: NativeFrameFunction = NativeFrameFunction {
  make_frame: || Box::new(FindFrame {
    this: None,
    this_arg: Val::Undefined,
    condition_fn: Val::Void,
    param_i: 0,
    array_i: 0,
    found: None,
  }),
};

struct FindFrame {
  this: Option<Rc<VsArray>>,
  this_arg: Val,
  condition_fn: Val,
  param_i: usize,
  array_i: usize,
  found: Option<Val>,
}

impl StackFrameTrait for FindFrame {
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
      None => std::panic!("Not implemented: exception: find called on non-array"),
      Some(ad) => ad,
    };

    for res in &self.found {
      return FrameStepResult::Pop(CallResult {
        return_: res.clone(),
        this: Val::Array(array_data.clone()),
      });
    }

    let array_i = self.array_i;
    self.array_i += 1;

    match array_data.elements.get(array_i) {
      Some(el) => match el {
        Val::Void => {
          return FrameStepResult::Continue;
        },
        _ => match self.condition_fn.load_function() {
          LoadFunctionResult::NotAFunction =>
            std::panic!("Not implemented: exception: find fn is not a function")
          ,
          LoadFunctionResult::NativeFunction(native_fn) => {
            let cond_result = native_fn(
              &mut self.this_arg.clone(),
              vec![
                el.clone(),
                Val::Number(array_i as f64),
                Val::Array(array_data.clone()),
              ],
            );

            if cond_result.is_truthy() {
              return FrameStepResult::Pop(CallResult {
                return_: el.clone(),
                this: Val::Array(array_data.clone()),
              });
            }
  
            return FrameStepResult::Continue;
          },
          LoadFunctionResult::StackFrame(mut new_frame) => {
            new_frame.write_this(self.this_arg.clone());
            new_frame.write_param(el.clone());
            new_frame.write_param(Val::Number(array_i as f64));
            new_frame.write_param(Val::Array(array_data.clone()));
            return FrameStepResult::Push(new_frame);
          },
        },
      },
      None => {
        return FrameStepResult::Pop(CallResult {
          return_: Val::Undefined,
          this: Val::Array(array_data.clone()),
        });
      },
    };
  }

  fn apply_call_result(&mut self, call_result: CallResult) {
    if call_result.return_.is_truthy() {
      let array_data = match &self.this {
        None => std::panic!("Not implemented: exception: find called on non-array"),
        Some(ad) => ad,
      };

      self.found = Some(array_data.elements[self.array_i - 1].clone());
    }
  }

  fn get_call_result(&mut self) -> CallResult {
    std::panic!("Not appropriate for FindFrame")
  }
}
