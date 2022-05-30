use std::rc::Rc;

use super::super::vs_value::{Val, ValTrait, LoadFunctionResult};
use super::super::vs_array::VsArray;
use super::super::native_frame_function::NativeFrameFunction;
use super::super::stack_frame::{StackFrameTrait, FrameStepResult, CallResult};

pub static MAP: NativeFrameFunction = NativeFrameFunction {
  make_frame: || Box::new(MapFrame {
    this: None,
    mapper: Val::Void,
    param_i: 0,
    map_results: Vec::new(),
  }),
};

struct MapFrame {
  this: Option<Rc<VsArray>>,
  mapper: Val,
  param_i: usize,
  map_results: Vec<Val>,
}

impl StackFrameTrait for MapFrame {
  fn write_this(&mut self, this: Val) {
    self.this = this.as_array_data();
  }

  fn write_param(&mut self, param: Val) {
    if self.param_i == 0 {
      self.mapper = param;
    }

    self.param_i += 1;
  }

  fn step(&mut self) -> FrameStepResult {
    let array_data = match &self.this {
      None => std::panic!("Not implemented: exception: map called on non-array"),
      Some(ad) => ad,
    };

    match array_data.elements.get(self.map_results.len()) {
      Some(el) => match self.mapper.load_function() {
        LoadFunctionResult::NotAFunction =>
          std::panic!("Not implemented: exception: map fn is not a function")
        ,
        LoadFunctionResult::NativeFunction(native_fn) => {
          self.map_results.push(native_fn(
            &mut Val::Undefined,
            vec![el.clone()],
          ));

          return FrameStepResult::Continue;
        },
        LoadFunctionResult::StackFrame(mut new_frame) => {
          new_frame.write_param(el.clone());
          return FrameStepResult::Push(new_frame);
        },
      },
      None => {
        let mut return_elements = Vec::new();
        std::mem::swap(&mut return_elements, &mut self.map_results);
  
        return FrameStepResult::Pop(CallResult {
          return_: Val::Array(Rc::new(VsArray::from(return_elements))),
          this: Val::Array(array_data.clone()),
        });
      },
    };
  }

  fn apply_call_result(&mut self, call_result: CallResult) {
    self.map_results.push(call_result.return_);
  }

  fn get_call_result(&mut self) -> CallResult {
    std::panic!("Not appropriate for MapFrame")
  }
}
