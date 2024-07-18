use std::mem::take;
use std::rc::Rc;

use crate::builtins::internal_error_builtin::ToInternalError;
use crate::bytecode::Bytecode;
use crate::bytecode::DecoderMaker;
use crate::first_stack_frame::FirstStackFrame;
use crate::stack_frame::CallResult;
use crate::stack_frame::FrameStepOk;
use crate::stack_frame::StackFrame;
use crate::vs_value::{LoadFunctionResult, Val, ValTrait};

pub struct VirtualMachine {
  pub frame: StackFrame,
  pub stack: Vec<StackFrame>,
}

impl Default for VirtualMachine {
  fn default() -> Self {
    VirtualMachine {
      frame: Box::new(FirstStackFrame::new()),
      stack: Default::default(),
    }
  }
}

impl VirtualMachine {
  pub fn run(
    &mut self,
    step_limit: Option<usize>,
    this: &mut Val,
    fn_: Val,
    args: Vec<Val>,
  ) -> Result<Val, Val> {
    let mut frame = match fn_.load_function() {
      LoadFunctionResult::StackFrame(f) => f,
      _ => return Err("fn_ is not a function".to_internal_error()),
    };

    frame.write_this(false, take(this))?;

    for a in args {
      frame.write_param(a);
    }

    self.push(frame);

    let res = match step_limit {
      Some(step_limit) => 'b: {
        let mut step_count = 0;

        while step_count < step_limit {
          self.step()?;
          step_count += 1;

          if self.stack.is_empty() {
            break 'b self.frame.get_call_result();
          }
        }

        return Err("step limit reached".to_internal_error());
      }
      None => {
        while !self.stack.is_empty() {
          self.step()?;
        }

        self.frame.get_call_result()
      }
    };

    let CallResult {
      return_,
      this: updated_this,
    } = res;

    *this = updated_this;

    Ok(return_)
  }

  pub fn step(&mut self) -> Result<(), Val> {
    let step_ok = match self.frame.step() {
      Ok(step_ok) => step_ok,
      Err(e) => return self.handle_exception(e),
    };

    match step_ok {
      FrameStepOk::Continue => {}
      FrameStepOk::Pop(call_result) => {
        self.pop();
        self.frame.apply_call_result(call_result);
      }
      FrameStepOk::Push(new_frame) => {
        self.push(new_frame);
      }
      // TODO: Internal errors
      FrameStepOk::Yield(_) => {
        return self.handle_exception("Unexpected yield".to_internal_error())
      }
      FrameStepOk::YieldStar(_) => {
        return self.handle_exception("Unexpected yield*".to_internal_error())
      }
    }

    Ok(())
  }

  pub fn push(&mut self, mut frame: StackFrame) {
    std::mem::swap(&mut self.frame, &mut frame);
    self.stack.push(frame);
  }

  pub fn pop(&mut self) {
    // This name is accurate after the swap
    let mut old_frame = self.stack.pop().unwrap();
    std::mem::swap(&mut self.frame, &mut old_frame);
  }

  pub fn handle_exception(&mut self, mut exception: Val) -> Result<(), Val> {
    while !self.stack.is_empty() {
      if self.frame.can_catch_exception(&exception) {
        self.frame.catch_exception(&mut exception);
        return Ok(());
      }

      if self.stack.is_empty() {
        return Err(exception);
      }

      self.pop();
    }

    Err(exception)
  }

  pub fn read_default_export(bytecode: Rc<Bytecode>) -> Val {
    bytecode.decoder(0).decode_val(&mut Vec::new())
  }
}
