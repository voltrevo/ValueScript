use crate::stack_frame::StackFrame;

use super::stack_frame::{CallResult, FrameStepResult, StackFrameTrait};
use super::vs_value::Val;

#[derive(Clone)]
pub struct FirstStackFrame {
  call_result: CallResult,
}

impl FirstStackFrame {
  pub fn new() -> FirstStackFrame {
    FirstStackFrame {
      call_result: CallResult {
        return_: Val::Void,
        this: Val::Void,
      },
    }
  }
}

impl StackFrameTrait for FirstStackFrame {
  fn write_this(&mut self, _const: bool, _this: Val) -> Result<(), Val> {
    panic!("Not appropriate for FirstStackFrame");
  }

  fn write_param(&mut self, _param: Val) {
    panic!("Not appropriate for FirstStackFrame");
  }

  fn step(&mut self) -> FrameStepResult {
    panic!("Not appropriate for FirstStackFrame");
  }

  fn apply_call_result(&mut self, call_result: CallResult) {
    self.call_result = call_result;
  }

  fn get_call_result(&mut self) -> CallResult {
    // TODO: get_call_result(self) version? (Move memory variation)
    self.call_result.clone()
  }

  fn catch_exception(&mut self, _exception: &mut Val) {
    panic!("Not appropriate for FirstStackFrame");
  }

  fn clone_to_stack_frame(&self) -> StackFrame {
    Box::new(self.clone())
  }
}
