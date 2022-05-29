use super::vs_value::Val;

#[derive(Clone)]
pub struct CallResult {
  pub return_: Val,
  pub this: Val,
}

pub enum FrameStepResult {
  Continue,
  Pop(CallResult),
  Push(Box<dyn StackFrameTrait>),
}

pub trait StackFrameTrait {
  fn write_this(&mut self, this: Val);
  fn write_param(&mut self, param: Val);
  fn step(&mut self) -> FrameStepResult;
  fn apply_call_result(&mut self, call_result: CallResult);
  fn get_call_result(&mut self) -> CallResult;
}
