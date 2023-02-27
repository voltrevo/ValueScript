use std::rc::Rc;

use super::super::vs_value::{Val, ValTrait};
use super::super::vs_array::VsArray;
use super::super::native_frame_function::NativeFrameFunction;
use super::array_mapping_frame::{ArrayMappingState, ArrayMappingFrame};

pub static FILTER: NativeFrameFunction = NativeFrameFunction {
  make_frame: || Box::new(ArrayMappingFrame::new(Box::new(FilterState::default()))),
};

#[derive(Default)]
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
    return Val::Array(Rc::new(VsArray::from(filter_results)));
  }
}
