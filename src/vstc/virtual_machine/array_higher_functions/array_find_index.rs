use super::super::vs_value::{Val, ValTrait};
use super::super::native_frame_function::NativeFrameFunction;
use super::array_mapping_frame::{ArrayMappingState, ArrayMappingFrame};

pub static FIND_INDEX: NativeFrameFunction = NativeFrameFunction {
  make_frame: || Box::new(ArrayMappingFrame::new(Box::new(FindIndexState::default()))),
};

#[derive(Default)]
struct FindIndexState {}

impl ArrayMappingState for FindIndexState {
  fn process(&mut self, i: usize, _element: &Val, mapped: Val) -> Option<Val> {
    match mapped.is_truthy() {
      true => Some(Val::Number(i as f64)),
      false => None,
    }
  }

  fn finish(&mut self) -> Val {
    Val::Number(-1f64)
  }
}
