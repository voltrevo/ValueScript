use std::{mem::take, rc::Rc};

use crate::{
  builtins::{internal_error_builtin::ToInternalError, type_error_builtin::ToTypeError},
  native_function::ThisWrapper,
  operations::op_sub,
  stack_frame::{CallResult, FrameStepOk, FrameStepResult, StackFrame, StackFrameTrait},
  vs_symbol::VsSymbol,
  vs_value::{ToVal, Val},
  LoadFunctionResult, ValTrait,
};

#[derive(Debug, Clone)]
pub struct CatStackFrame {
  pub state: CatFrameState,
  pub iter_result: Option<Val>,
  pub args: Vec<Val>,
  pub i: usize,
  pub res: Vec<Val>,
}

#[derive(Debug, Clone)]
pub enum CatFrameState {
  ReadNext,
  MakingIterator,
  Iterating(Val),
}

impl CatStackFrame {
  pub fn from_args(args: Vec<Val>) -> Self {
    Self {
      state: CatFrameState::ReadNext,
      iter_result: None,
      args,
      i: 0,
      res: vec![],
    }
  }

  fn read_next(&mut self) -> FrameStepResult {
    let mut arg = match self.args.get_mut(self.i) {
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

    let make_iter = op_sub(&mut arg, &VsSymbol::ITERATOR.to_val())?;

    match make_iter.load_function() {
      LoadFunctionResult::NotAFunction => Err("Non-iterable cat argument".to_type_error()),
      LoadFunctionResult::NativeFunction(fn_) => {
        self.state = CatFrameState::Iterating(fn_(ThisWrapper::new(true, &mut arg), vec![])?);
        Ok(FrameStepOk::Continue)
      }
      LoadFunctionResult::StackFrame(mut new_frame) => {
        self.state = CatFrameState::MakingIterator;
        new_frame.write_this(true, arg)?;
        Ok(FrameStepOk::Push(new_frame))
      }
    }
  }

  fn apply_iter_result(
    state: &mut CatFrameState,
    res: &mut Vec<Val>,
    iter_result: Val,
  ) -> Result<(), Val> {
    let done = iter_result.sub(&"done".to_val())?.is_truthy();

    if done {
      *state = CatFrameState::ReadNext;
    } else {
      res.push(iter_result.sub(&"value".to_val())?);
    }

    Ok(())
  }
}

impl StackFrameTrait for CatStackFrame {
  fn write_this(&mut self, _const: bool, _this: Val) -> Result<(), Val> {
    Ok(())
  }

  fn write_param(&mut self, param: Val) {
    self.args.push(param);
  }

  fn step(&mut self) -> FrameStepResult {
    if let Some(iter_result) = take(&mut self.iter_result) {
      Self::apply_iter_result(&mut self.state, &mut self.res, iter_result)?;
    }

    match &mut self.state {
      CatFrameState::ReadNext => self.read_next(),
      CatFrameState::MakingIterator => {
        Err("Unexpected step during MakingIterator".to_internal_error())
      }
      CatFrameState::Iterating(iter) => match iter.sub(&"next".to_val())?.load_function() {
        LoadFunctionResult::NotAFunction => Err(".next was not a function".to_type_error()),
        LoadFunctionResult::NativeFunction(fn_) => {
          let iter_result = fn_(ThisWrapper::new(false, iter), vec![])?;
          Self::apply_iter_result(&mut self.state, &mut self.res, iter_result)?;
          Ok(FrameStepOk::Continue)
        }
        LoadFunctionResult::StackFrame(mut new_frame) => {
          new_frame.write_this(false, iter.clone())?;
          Ok(FrameStepOk::Push(new_frame))
        }
      },
    }
  }

  fn apply_call_result(&mut self, call_result: CallResult) {
    match &mut self.state {
      CatFrameState::ReadNext => panic!("Unexpected call result during ReadNext"),
      CatFrameState::MakingIterator => self.state = CatFrameState::Iterating(call_result.return_),
      CatFrameState::Iterating(iter) => {
        *iter = call_result.this;
        self.iter_result = Some(call_result.return_);
      }
    }
  }

  fn get_call_result(&mut self) -> CallResult {
    panic!("Not appropriate for CatStackFrame");
  }

  fn catch_exception(&mut self, _exception: &mut Val) {}

  fn clone_to_stack_frame(&self) -> StackFrame {
    Box::new(self.clone())
  }
}
