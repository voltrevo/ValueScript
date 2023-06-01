use super::vs_value::Val;

pub type StackFrame = Box<dyn StackFrameTrait>;

#[derive(Clone)]
pub struct CallResult {
  pub return_: Val,
  pub this: Val,
}

pub enum FrameStepOk {
  Continue,
  Pop(CallResult),
  Push(StackFrame),
  Yield(Val),
  YieldStar(Val),
}

pub type FrameStepResult = Result<FrameStepOk, Val>;

pub trait StackFrameTrait {
  fn write_this(&mut self, const_: bool, this: Val) -> Result<(), Val>;
  fn write_param(&mut self, param: Val);
  fn step(&mut self) -> FrameStepResult;
  fn apply_call_result(&mut self, call_result: CallResult);
  fn get_call_result(&mut self) -> CallResult;
  fn catch_exception(&mut self, exception: Val) -> bool;
  fn clone_to_stack_frame(&self) -> StackFrame;
}

impl Clone for StackFrame {
  fn clone(&self) -> Self {
    self.clone_to_stack_frame()
  }
}

impl Default for StackFrame {
  fn default() -> Self {
    Box::new(VoidStackFrame {})
  }
}

#[derive(Clone)]
struct VoidStackFrame {}

impl StackFrameTrait for VoidStackFrame {
  fn write_this(&mut self, _const: bool, _this: Val) -> Result<(), Val> {
    Ok(())
  }

  fn write_param(&mut self, _param: Val) {}

  fn step(&mut self) -> FrameStepResult {
    Ok(FrameStepOk::Continue)
  }

  fn apply_call_result(&mut self, _call_result: CallResult) {}

  fn get_call_result(&mut self) -> CallResult {
    CallResult {
      return_: Val::Void,
      this: Val::Void,
    }
  }

  fn catch_exception(&mut self, _exception: Val) -> bool {
    false
  }

  fn clone_to_stack_frame(&self) -> StackFrame {
    Box::new(self.clone())
  }
}
