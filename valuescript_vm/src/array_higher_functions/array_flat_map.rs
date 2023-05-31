use crate::vs_value::ToVal;

use super::super::native_frame_function::NativeFrameFunction;
use super::super::vs_value::{Val, ValTrait};
use super::array_mapping_frame::{ArrayMappingFrame, ArrayMappingState};

pub static FLAT_MAP: NativeFrameFunction = NativeFrameFunction {
  make_frame: || Box::new(ArrayMappingFrame::new(Box::new(FlatMapState::default()))),
};

#[derive(Default, Clone)]
struct FlatMapState {
  flat_map_results: Vec<Val>,
}

impl ArrayMappingState for FlatMapState {
  fn process(&mut self, _i: usize, _element: &Val, mapped: Val) -> Option<Val> {
    match mapped.as_array_data() {
      None => self.flat_map_results.push(mapped),
      Some(array_data) => {
        for el in &array_data.elements {
          self.flat_map_results.push(el.clone());
        }
      }
    }

    return None;
  }

  fn finish(&mut self) -> Val {
    let mut flat_map_results = Vec::new();
    std::mem::swap(&mut self.flat_map_results, &mut flat_map_results);
    flat_map_results.to_val()
  }

  fn clone_to_array_mapping_state(&self) -> Box<dyn ArrayMappingState> {
    Box::new(self.clone())
  }
}
