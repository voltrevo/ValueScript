use std::rc::Rc;

use super::vs_value::{Val, ValTrait, LoadFunctionResult};
use super::bytecode_decoder::BytecodeDecoder;
use super::stack_frame::{StackFrame, FrameStepResult};
use super::first_stack_frame::FirstStackFrame;

pub struct VirtualMachine {
  pub frame: StackFrame,
  pub stack: Vec<StackFrame>,
}

impl VirtualMachine {
  pub fn run(&mut self, bytecode: &Rc<Vec<u8>>, params: &[String]) -> Val {
    let mut bd = BytecodeDecoder {
      data: bytecode.clone(),
      pos: 0,
    };

    let main_fn = bd.decode_val(&Vec::new());

    let mut frame = match main_fn.load_function() {
      LoadFunctionResult::StackFrame(f) => f,
      _ => std::panic!("bytecode does start with function"),
    };

    for p in params {
      frame.write_param(Val::String(Rc::new(p.clone())));
    }

    self.push(frame);

    while self.stack.len() > 0 {
      self.step();
    }

    return self.frame.get_call_result().return_;
  }

  pub fn new() -> VirtualMachine {
    let mut registers: Vec<Val> = Vec::with_capacity(2);
    registers.push(Val::Undefined);
    registers.push(Val::Undefined);

    return VirtualMachine {
      frame: Box::new(FirstStackFrame::new()),
      stack: Default::default(),
    };
  }

  pub fn step(&mut self) {
    match self.frame.step() {
      FrameStepResult::Continue => {},
      FrameStepResult::Pop(call_result) => {
        self.pop();
        self.frame.apply_call_result(call_result);
      },
      FrameStepResult::Push(new_frame) => {
        self.push(new_frame);
      },
    };
  }

  pub fn push(&mut self, mut frame: StackFrame) {
    std::mem::swap(&mut self.frame, &mut frame);
    self.stack.push(frame);
  }

  pub fn pop(&mut self) {
    // This name is accurate after the swap
    let mut old_frame = self.stack.pop().unwrap();
    std::mem::swap(&mut self.frame, &mut old_frame);
  }
}
