use std::rc::Rc;

use super::vs_value::VsType;
use super::vs_value::VsValue;
use super::vs_value::Val;
use super::vs_undefined::VsUndefined;
use super::virtual_machine::VirtualMachine;
use super::virtual_machine::StackFrame;
use super::bytecode_decoder::BytecodeDecoder;

pub struct VsFunction {
  pub bytecode: Rc<Vec<u8>>,
  pub register_count: usize,
  pub parameter_count: usize,
  pub start: usize,
}

impl VsValue for VsFunction {
  fn typeof_(&self) -> VsType {
    return VsType::Function;
  }

  fn to_string(&self) -> String {
    return "[function]".to_string();
  }

  fn to_number(&self) -> f64 {
    return f64::NAN;
  }

  fn is_primitive(&self) -> bool {
    return false;
  }

  fn push_frame(&self, vm: &mut VirtualMachine) -> bool {
    let mut registers: Vec<Val> = Vec::with_capacity(self.register_count - 1);
    
    for i in 0..(self.register_count - 1) {
      registers[i] = VsUndefined::new();
    }

    vm.stack.push(StackFrame {
      decoder: BytecodeDecoder {
        data: self.bytecode.clone(),
        pos: self.start,
      },
      registers: registers,
      this_target: 0,
      return_target: 0,
    });

    return true;
  }
}
