use super::super::vs_value::{Val, ValTrait};
use super::super::native_frame_function::NativeFrameFunction;
use super::array_mapping_frame::{ArrayMappingState, ArrayMappingFrame};

pub static SOME: NativeFrameFunction = NativeFrameFunction {
  make_frame: || Box::new(ArrayMappingFrame::new(Box::new(SomeState::default()))),
};

#[derive(Default)]
struct SomeState {}

impl ArrayMappingState for SomeState {
  fn process(&mut self, _i: usize, _element: &Val, mapped: Val) -> Option<Val> {
    match mapped.is_truthy() {
      true => Some(Val::Bool(true)),
      false => None,
    }
  }

  fn finish(&mut self) -> Val {
    Val::Bool(false)
  }
}
