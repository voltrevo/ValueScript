use super::super::native_frame_function::NativeFrameFunction;
use super::super::vs_value::{Val, ValTrait};
use super::array_mapping_frame::{ArrayMappingFrame, ArrayMappingState};

pub static EVERY: NativeFrameFunction = NativeFrameFunction {
  make_frame: || Box::new(ArrayMappingFrame::new(Box::new(EveryState::default()))),
};

#[derive(Default, Clone)]
struct EveryState {}

impl ArrayMappingState for EveryState {
  fn process(&mut self, _i: usize, _element: &Val, mapped: Val) -> Option<Val> {
    match mapped.is_truthy() {
      true => None,
      false => Some(Val::Bool(false)),
    }
  }

  fn finish(&mut self) -> Val {
    Val::Bool(true)
  }

  fn clone_to_array_mapping_state(&self) -> Box<dyn ArrayMappingState> {
    Box::new(self.clone())
  }
}
