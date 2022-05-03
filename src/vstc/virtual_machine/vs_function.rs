use std::rc::Rc;

use super::vs_value::VsType;
use super::vs_value::Val;
use super::vs_value::ValTrait;
use super::virtual_machine::StackFrame;
use super::bytecode_decoder::BytecodeDecoder;

pub struct VsFunction {
  pub bytecode: Rc<Vec<u8>>,
  pub register_count: usize,
  pub parameter_count: usize,
  pub start: usize,
}

impl ValTrait for VsFunction {
  fn typeof_(&self) -> VsType {
    return VsType::Function;
  }

  fn val_to_string(&self) -> String {
    return "[function]".to_string();
  }

  fn to_number(&self) -> f64 {
    return f64::NAN;
  }

  fn is_primitive(&self) -> bool {
    return false;
  }

  fn to_primitive(&self) -> Val {
    return Val::String(Rc::new(self.val_to_string()));
  }

  fn make_frame(&self) -> Option<StackFrame> {
    let mut registers: Vec<Val> = Vec::with_capacity(self.register_count - 1);
    
    for _ in 0..(self.register_count - 1) {
      registers.push(Val::Undefined);
    }

    return Some(StackFrame {
      decoder: BytecodeDecoder {
        data: self.bytecode.clone(),
        pos: self.start,
      },
      registers: registers,
      this_target: None,
      return_target: None,
    });
  }

  fn is_truthy(&self) -> bool {
    return true;
  }
}
