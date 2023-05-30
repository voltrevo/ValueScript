use std::{mem::take, rc::Rc};

use crate::{
  builtins::error_builtin::ToError,
  stack_frame::{CallResult, FrameStepOk, FrameStepResult, StackFrameTrait},
  vs_value::{ToVal, Val},
};

#[derive(Debug)]
pub struct CatStackFrame {
  pub args: Vec<Val>,
  pub i: usize,
  pub res: Vec<Val>,
}

impl StackFrameTrait for CatStackFrame {
  fn write_this(&mut self, _const: bool, _this: Val) -> Result<(), Val> {
    Ok(())
  }

  fn write_param(&mut self, param: Val) {
    self.args.push(param);
  }

  fn step(&mut self) -> FrameStepResult {
    let arg = match self.args.get_mut(self.i) {
      None => {
        return Ok(FrameStepOk::Pop(CallResult {
          return_: take(&mut self.res).to_val(),
          this: Val::Undefined,
        }));
      }
      Some(arg) => take(arg),
    };

    self.i += 1;

    if let Val::Array(mut arg) = arg {
      match Rc::get_mut(&mut arg) {
        Some(arg) => self.res.append(&mut arg.elements),
        None => {
          for item in &arg.elements {
            self.res.push(item.clone());
          }
        }
      }

      return Ok(FrameStepOk::Continue);
    }

    Err("TODO: Cat: Non-array iterables".to_error())
  }

  fn apply_call_result(&mut self, _call_result: CallResult) {
    panic!("Not expected (yet)");
  }

  fn get_call_result(&mut self) -> CallResult {
    panic!("Not appropriate for CatStackFrame");
  }

  fn catch_exception(&mut self, _exception: Val) -> bool {
    false
  }
}
