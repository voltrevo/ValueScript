use std::rc::Rc;

use crate::builtins::type_error_builtin::ToTypeError;
use crate::native_function::ThisWrapper;
use crate::stack_frame::{CallResult, FrameStepOk, StackFrameTrait};
use crate::stack_frame::{FrameStepResult, StackFrame};
use crate::vs_array::VsArray;
use crate::vs_value::{LoadFunctionResult, Val, ValTrait};

pub trait ArrayMappingState {
  fn process(&mut self, i: usize, element: &Val, mapped: Val) -> Option<Val>;
  fn finish(&mut self) -> Val;
  fn clone_to_array_mapping_state(&self) -> Box<dyn ArrayMappingState>;
}

impl Clone for Box<dyn ArrayMappingState> {
  fn clone(&self) -> Self {
    self.clone_to_array_mapping_state()
  }
}

#[derive(Clone)]
pub struct ArrayMappingFrame {
  state: Box<dyn ArrayMappingState>,
  early_exit: Option<Result<Val, Val>>,

  this: Option<Rc<VsArray>>,
  array_i: usize,

  mapper: Val,
  this_arg: Val,
  param_i: usize,
}

impl ArrayMappingFrame {
  pub fn new(state: Box<dyn ArrayMappingState>) -> ArrayMappingFrame {
    return ArrayMappingFrame {
      state: state,
      early_exit: None,
      this: None,
      array_i: 0,
      mapper: Val::Void,
      this_arg: Val::Undefined,
      param_i: 0,
    };
  }
}

impl StackFrameTrait for ArrayMappingFrame {
  fn write_this(&mut self, _const: bool, this: Val) -> Result<(), Val> {
    self.this = this.as_array_data();
    Ok(())
  }

  fn write_param(&mut self, param: Val) {
    match self.param_i {
      0 => {
        self.mapper = param;
      }
      1 => {
        self.this_arg = param;
      }
      _ => {}
    };

    self.param_i += 1;
  }

  fn step(&mut self) -> FrameStepResult {
    let array_data = match &self.this {
      None => return Err("Array fn called on non-array".to_type_error()),
      Some(ad) => ad,
    };

    if let Some(early_exit) = &self.early_exit {
      let early_exit = early_exit.clone()?;

      return Ok(FrameStepOk::Pop(CallResult {
        return_: early_exit,
        this: Val::Array(array_data.clone()),
      }));
    }

    let array_i = self.array_i;
    self.array_i += 1;

    match array_data.elements.get(array_i) {
      Some(el) => match el {
        Val::Void => Ok(FrameStepOk::Continue),
        _ => match self.mapper.load_function() {
          LoadFunctionResult::NotAFunction => Err("map fn is not a function".to_type_error()),
          LoadFunctionResult::NativeFunction(native_fn) => {
            match self.state.process(
              array_i,
              el,
              native_fn(
                ThisWrapper::new(false, &mut self.this_arg.clone()),
                vec![
                  el.clone(),
                  Val::Number(array_i as f64),
                  Val::Array(array_data.clone()),
                ],
              )?,
            ) {
              None => Ok(FrameStepOk::Continue),
              Some(val) => Ok(FrameStepOk::Pop(CallResult {
                return_: val,
                this: Val::Array(array_data.clone()),
              })),
            }
          }
          LoadFunctionResult::StackFrame(mut new_frame) => {
            new_frame.write_this(true, self.this_arg.clone())?;
            new_frame.write_param(el.clone());
            new_frame.write_param(Val::Number(array_i as f64));
            new_frame.write_param(Val::Array(array_data.clone()));
            Ok(FrameStepOk::Push(new_frame))
          }
        },
      },
      None => Ok(FrameStepOk::Pop(CallResult {
        return_: self.state.finish(),
        this: Val::Array(array_data.clone()),
      })),
    }
  }

  fn apply_call_result(&mut self, call_result: CallResult) {
    let array_i = self.array_i - 1;

    let element = match &self.this {
      None => {
        self.early_exit = Some(Err("Array fn called on non-array".to_type_error()));
        return;
      }
      Some(ad) => &ad.elements[array_i],
    };

    self.early_exit = self
      .state
      .process(array_i, element, call_result.return_)
      .map(|v| Ok(v));
  }

  fn get_call_result(&mut self) -> CallResult {
    panic!("Not appropriate for MapFrame")
  }

  fn catch_exception(&mut self, _exception: Val) -> bool {
    return false;
  }

  fn clone_to_stack_frame(&self) -> StackFrame {
    Box::new(self.clone())
  }
}
