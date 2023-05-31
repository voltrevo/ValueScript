use crate::vs_value::ToVal;

use super::super::native_frame_function::NativeFrameFunction;
use super::super::vs_value::{Val, ValTrait};
use super::array_mapping_frame::{ArrayMappingFrame, ArrayMappingState};

pub static FILTER: NativeFrameFunction = NativeFrameFunction {
  make_frame: || Box::new(ArrayMappingFrame::new(Box::new(FilterState::default()))),
};

#[derive(Default, Clone)]
struct FilterState {
  filter_results: Vec<Val>,
}

impl ArrayMappingState for FilterState {
  fn process(&mut self, _i: usize, element: &Val, mapped: Val) -> Option<Val> {
    if mapped.is_truthy() {
      self.filter_results.push(element.clone());
    }

    return None;
  }

  fn finish(&mut self) -> Val {
    let mut filter_results = Vec::new();
    std::mem::swap(&mut self.filter_results, &mut filter_results);
    filter_results.to_val()
  }

  fn clone_to_array_mapping_state(&self) -> Box<dyn ArrayMappingState> {
    Box::new(self.clone())
  }
}
