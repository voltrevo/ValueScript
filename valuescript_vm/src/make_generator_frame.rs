use std::mem::take;

use crate::{
  bytecode_stack_frame::BytecodeStackFrame,
  generator::Generator,
  stack_frame::{CallResult, FrameStepOk, FrameStepResult, StackFrame, StackFrameTrait},
  vs_value::{ToDynamicVal, Val},
};

#[derive(Clone)]
pub struct MakeGeneratorFrame {
  pub frame: Option<BytecodeStackFrame>,
}

impl MakeGeneratorFrame {
  pub fn new(frame: BytecodeStackFrame) -> MakeGeneratorFrame {
    return MakeGeneratorFrame { frame: Some(frame) };
  }

  fn frame_mut(&mut self) -> &mut BytecodeStackFrame {
    self.frame.as_mut().unwrap()
  }

  fn take_frame(&mut self) -> BytecodeStackFrame {
    take(&mut self.frame).unwrap()
  }
}

impl StackFrameTrait for MakeGeneratorFrame {
  fn write_this(&mut self, const_: bool, this: Val) -> Result<(), Val> {
    self.frame_mut().write_this(const_, this)
  }

  fn write_param(&mut self, param: Val) {
    self.frame_mut().write_param(param);
  }

  fn step(&mut self) -> FrameStepResult {
    Ok(FrameStepOk::Pop(CallResult {
      return_: Generator::new(Box::new(self.take_frame())).to_dynamic_val(),
      this: Val::Undefined,
    }))
  }

  fn apply_call_result(&mut self, _call_result: CallResult) {
    panic!("Not appropriate for MakeGeneratorFrame");
  }

  fn get_call_result(&mut self) -> CallResult {
    panic!("Not appropriate for MakeGeneratorFrame")
  }

  fn catch_exception(&mut self, _exception: &mut Val) {
    panic!("Not appropriate for MakeGeneratorFrame");
  }

  fn clone_to_stack_frame(&self) -> StackFrame {
    Box::new(self.clone())
  }
}
