use super::stack_frame::{StackFrameTrait, FrameStepResult, CallResult};
use super::vs_value::Val;

pub struct FirstStackFrame {
  call_result: CallResult,
}

impl FirstStackFrame {
  pub fn new() -> FirstStackFrame {
    return FirstStackFrame {
      call_result: CallResult {
        return_: Val::Void,
        this: Val::Void,
      },
    };
  }
}

impl StackFrameTrait for FirstStackFrame {
  fn write_this(&mut self, _this: Val) {
    std::panic!("Not appropriate for FirstStackFrame");
  }

  fn write_param(&mut self, _param: Val) {
    std::panic!("Not appropriate for FirstStackFrame");
  }

  fn step(&mut self) -> FrameStepResult {
    std::panic!("Not appropriate for FirstStackFrame");
  }

  fn apply_call_result(&mut self, call_result: CallResult) {
    self.call_result = call_result;
  }

  fn get_call_result(&mut self) -> CallResult {
    return self.call_result.clone();
  }
}
