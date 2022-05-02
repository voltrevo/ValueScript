use std::rc::Rc;

use super::vs_value::Val;
use super::vs_undefined::VsUndefined;
use super::vs_number::VsNumber;
use super::vs_string::VsString;
use super::operations;
use super::bytecode_decoder::BytecodeDecoder;
use super::instruction::Instruction;

pub struct VirtualMachine {
  pub return_value: Val,
  pub root: Val,
  pub stack: Vec<StackFrame>,
}

pub struct StackFrame {
  pub decoder: BytecodeDecoder,
  pub registers: Vec<Val>,
  pub this_target: usize,
  pub return_target: usize,
}

impl VirtualMachine {
  pub fn run(&mut self, bytecode: &Rc<Vec<u8>>) {
    let mut bd = BytecodeDecoder {
      data: bytecode.clone(),
      pos: 0,
    };

    let main_fn = bd.decode_val();

    if !main_fn.push_frame(self) {
      std::panic!("bytecode does start with function")
    }

    while self.stack.len() > 0 {
      self.step();
    }
  }

  pub fn new() -> VirtualMachine {
    return VirtualMachine {
      root: VsUndefined::new(),
      return_value: VsUndefined::new(),
      stack: Default::default(),
    };
  }

  pub fn step(&mut self) {
    use Instruction::*;

    let frame = self.stack.last_mut().unwrap();
    
    match frame.decoder.decode_instruction() {
      End => {
        self.pop();
      },

      Mov => {
        let val = frame.decoder.decode_val();
        let register_index = frame.decoder.decode_register();

        if register_index.is_some() {
          frame.registers[register_index.unwrap()] = val;
        }
      },

      OpInc => {
        let mut val = frame.decoder.decode_val();
        val = operations::op_plus(&val, &VsNumber::from_f64(1_f64));
        let register_index = frame.decoder.decode_register();

        if register_index.is_some() {
          frame.registers[register_index.unwrap()] = val;
        }
      },

      OpPlus => {
        let left = frame.decoder.decode_val();
        let right = frame.decoder.decode_val();

        let register_index = frame.decoder.decode_register();

        if register_index.is_some() {
          frame.registers[register_index.unwrap()] = operations::op_plus(&left, &right);
        }
      },

      OpMul => {
        let left = frame.decoder.decode_val();
        let right = frame.decoder.decode_val();

        let register_index = frame.decoder.decode_register();

        if register_index.is_some() {
          frame.registers[register_index.unwrap()] = operations::op_mul(&left, &right);
        }
      },

      OpMod => {
        let left = frame.decoder.decode_val();
        let right = frame.decoder.decode_val();

        let register_index = frame.decoder.decode_register();

        if register_index.is_some() {
          frame.registers[register_index.unwrap()] = operations::op_mod(&left, &right);
        }
      },

      // 0x11 => OpLess,
      // 0x27 => Jmp,
      // 0x28 => JmpIf,

      _ => std::panic!("Not implemented"),
    };
  }

  pub fn pop(&mut self) {
    let old_frame = self.stack.pop().unwrap();
    let optional_frame = self.stack.last_mut();

    if optional_frame.is_some() {
      let frame = optional_frame.unwrap();

      frame.registers[frame.return_target] = old_frame.registers[0].clone();
      frame.registers[frame.this_target] = old_frame.registers[1].clone();
    } else {
      // TODO: Use special init frame to avoid branching
      self.return_value = old_frame.registers[0].clone();
      self.root = old_frame.registers[1].clone();
    }
  }
}
