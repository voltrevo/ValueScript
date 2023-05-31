use super::super::native_frame_function::NativeFrameFunction;
use super::super::vs_value::{Val, ValTrait};
use super::array_mapping_frame::{ArrayMappingFrame, ArrayMappingState};

pub static FIND: NativeFrameFunction = NativeFrameFunction {
  make_frame: || Box::new(ArrayMappingFrame::new(Box::new(FindState::default()))),
};

#[derive(Default, Clone)]
struct FindState {}

impl ArrayMappingState for FindState {
  fn process(&mut self, _i: usize, element: &Val, mapped: Val) -> Option<Val> {
    match mapped.is_truthy() {
      true => Some(element.clone()),
      false => None,
    }
  }

  fn finish(&mut self) -> Val {
    Val::Undefined
  }

  fn clone_to_array_mapping_state(&self) -> Box<dyn ArrayMappingState> {
    Box::new(self.clone())
  }
}
