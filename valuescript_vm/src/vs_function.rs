use std::rc::Rc;

use super::vs_value::Val;
use super::bytecode_stack_frame::BytecodeStackFrame;
use super::bytecode_decoder::BytecodeDecoder;
use super::stack_frame::StackFrame;

pub struct VsFunction {
  pub bytecode: Rc<Vec<u8>>,
  pub register_count: usize,
  pub parameter_count: usize,
  pub start: usize,
  pub binds: Vec<Val>,
}

impl VsFunction {
  pub fn bind(&self, params: Vec<Val>) -> VsFunction {
    let mut new_binds = self.binds.clone();

    for p in params {
      new_binds.push(p);
    }

    return VsFunction {
      bytecode: self.bytecode.clone(),
      register_count: self.register_count,
      parameter_count: self.parameter_count,
      start: self.start,
      binds: new_binds,
    };
  }

  pub fn make_frame(&self) -> StackFrame {
    let mut registers: Vec<Val> = Vec::with_capacity(self.register_count - 1);

    registers.push(Val::Undefined);
    registers.push(Val::Undefined);

    for bind_val in &self.binds {
      registers.push(bind_val.clone());
    }

    while registers.len() < registers.capacity() {
      registers.push(Val::Undefined);
    }

    return Box::new(BytecodeStackFrame {
      decoder: BytecodeDecoder {
        data: self.bytecode.clone(),
        pos: self.start,
      },
      registers: registers,
      param_start: self.binds.len() + 2,
      param_end: self.parameter_count + 2,
      this_target: None,
      return_target: None,
    });
  }
}
