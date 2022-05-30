use super::super::vs_value::{Val, ValTrait};
use super::super::native_frame_function::NativeFrameFunction;
use super::array_mapping_frame::{ArrayMappingState, ArrayMappingFrame};

pub static FIND: NativeFrameFunction = NativeFrameFunction {
  make_frame: || Box::new(ArrayMappingFrame::new(Box::new(FindState::default()))),
};

#[derive(Default)]
struct FindState {}

impl ArrayMappingState for FindState {
  fn process(&mut self, _i: usize, element: &Val, mapped: Val) -> Option<Val> {
    match mapped.is_truthy() {
      true => None,
      false => Some(element.clone()),
    }
  }

  fn finish(&mut self) -> Val {
    Val::Undefined
  }
}
