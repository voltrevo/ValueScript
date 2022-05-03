use std::rc::Rc;

use super::vs_value::Val;
use super::vs_value::ValTrait;
use super::operations;
use super::bytecode_decoder::BytecodeDecoder;
use super::bytecode_decoder::BytecodeType;
use super::instruction::Instruction;

pub struct VirtualMachine {
  pub stack: Vec<StackFrame>,
}

pub struct StackFrame {
  pub decoder: BytecodeDecoder,
  pub registers: Vec<Val>,
  pub this_target: Option<usize>,
  pub return_target: Option<usize>,
}

impl VirtualMachine {
  pub fn run(&mut self, bytecode: &Rc<Vec<u8>>) -> Val {
    let mut bd = BytecodeDecoder {
      data: bytecode.clone(),
      pos: 0,
    };

    let main_fn = bd.decode_val(&Vec::new());
    let frame = main_fn.make_frame();

    if !frame.is_some() {
      std::panic!("bytecode does start with function")
    }

    self.stack.push(frame.unwrap());

    while self.stack.len() > 1 {
      self.step();
    }

    return self.stack[0].registers[0].clone();
  }

  pub fn new() -> VirtualMachine {
    let mut vm = VirtualMachine {
      stack: Default::default(),
    };

    let mut registers: Vec<Val> = Vec::with_capacity(2);
    registers.push(Val::Undefined);
    registers.push(Val::Undefined);

    let frame = StackFrame {
      decoder: BytecodeDecoder {
        data: Rc::new(Vec::new()),
        pos: 0,
      },
      registers: registers,
      return_target: Some(0),
      this_target: Some(1),
    };

    vm.stack.push(frame);

    return vm;
  }

  pub fn step(&mut self) {
    use Instruction::*;

    let mut frame = self.stack.last_mut().unwrap();

    match frame.decoder.decode_instruction() {
      End => {
        self.pop();
      },

      Mov => {
        let val = frame.decoder.decode_val(&frame.registers);
        let register_index = frame.decoder.decode_register_index();

        if register_index.is_some() {
          frame.registers[register_index.unwrap()] = val;
        }
      },

      OpInc => {
        let register_index = frame.decoder.decode_register_index().unwrap();
        let mut val = frame.registers[register_index].clone();
        val = operations::op_plus(&val, &Val::Number(1_f64));
        frame.registers[register_index] = val;
      },

      OpPlus => {
        let left = frame.decoder.decode_val(&frame.registers);
        let right = frame.decoder.decode_val(&frame.registers);

        let register_index = frame.decoder.decode_register_index();

        if register_index.is_some() {
          frame.registers[register_index.unwrap()] = operations::op_plus(&left, &right);
        }
      },

      OpMinus => {
        let left = frame.decoder.decode_val(&frame.registers);
        let right = frame.decoder.decode_val(&frame.registers);

        let register_index = frame.decoder.decode_register_index();

        if register_index.is_some() {
          frame.registers[register_index.unwrap()] = operations::op_minus(&left, &right);
        }
      },

      OpMul => {
        let left = frame.decoder.decode_val(&frame.registers);
        let right = frame.decoder.decode_val(&frame.registers);

        let register_index = frame.decoder.decode_register_index();

        if register_index.is_some() {
          frame.registers[register_index.unwrap()] = operations::op_mul(&left, &right);
        }
      },

      OpMod => {
        let left = frame.decoder.decode_val(&frame.registers);
        let right = frame.decoder.decode_val(&frame.registers);

        let register_index = frame.decoder.decode_register_index();

        if register_index.is_some() {
          frame.registers[register_index.unwrap()] = operations::op_mod(&left, &right);
        }
      },

      OpLess => {
        let left = frame.decoder.decode_val(&frame.registers);
        let right = frame.decoder.decode_val(&frame.registers);

        let register_index = frame.decoder.decode_register_index();

        if register_index.is_some() {
          frame.registers[register_index.unwrap()] = operations::op_less(&left, &right);
        }
      }

      OpTripleNe => {
        let left = frame.decoder.decode_val(&frame.registers);
        let right = frame.decoder.decode_val(&frame.registers);

        let register_index = frame.decoder.decode_register_index();

        if register_index.is_some() {
          frame.registers[register_index.unwrap()] = operations::op_triple_ne(&left, &right);
        }
      }

      Call => {
        let fn_ = frame.decoder.decode_val(&frame.registers);
        let maybe_new_frame = fn_.make_frame();

        if maybe_new_frame.is_none() {
          std::panic!("Not implemented: throw exception (fn_ is not a function)");
        }

        let mut new_frame = maybe_new_frame.unwrap();
        load_parameters(&mut frame, &mut new_frame);

        frame.return_target = frame.decoder.decode_register_index();
        frame.this_target = None;

        self.stack.push(new_frame);
      }

      Apply => {
        let fn_ = frame.decoder.decode_val(&frame.registers);
        let maybe_new_frame = fn_.make_frame();

        if maybe_new_frame.is_none() {
          std::panic!("Not implemented: throw exception (fn_ is not a function)");
        }

        let mut new_frame = maybe_new_frame.unwrap();

        if frame.decoder.peek_type() == BytecodeType::Register {
          frame.decoder.decode_type();
          let this_target = frame.decoder.decode_register_index();
          frame.this_target = this_target;

          if this_target.is_some() {
            new_frame.registers[1] = frame.registers[this_target.unwrap()].clone();
          }
        } else {
          frame.this_target = None;
          new_frame.registers[1] = frame.decoder.decode_val(&frame.registers);
        }

        load_parameters(&mut frame, &mut new_frame);

        frame.return_target = frame.decoder.decode_register_index();

        self.stack.push(new_frame);
      }

      Jmp => {
        let dst = frame.decoder.decode_pos();
        frame.decoder.pos = dst;
      }

      JmpIf => {
        let cond = frame.decoder.decode_val(&frame.registers);
        let dst = frame.decoder.decode_pos();

        if cond.is_truthy() {
          frame.decoder.pos = dst;
        }
      }

      _ => std::panic!("Not implemented"),
    };
  }

  pub fn pop(&mut self) {
    let old_frame = self.stack.pop().unwrap();
    let frame = self.stack.last_mut().unwrap();

    if frame.return_target.is_some() {
      frame.registers[frame.return_target.unwrap()] = old_frame.registers[0].clone();
    }

    if frame.this_target.is_some() {
      frame.registers[frame.this_target.unwrap()] = old_frame.registers[1].clone();
    }
  }
}

fn load_parameters(
  frame: &mut StackFrame,
  new_frame: &mut StackFrame,
) {
  let bytecode_type = frame.decoder.decode_type();

  if bytecode_type != BytecodeType::Array {
    std::panic!("Not implemented: call instruction not using inline array");
  }

  // Params start at 2 since 0:return, 1:this
  let mut reg_i = 2;

  while frame.decoder.peek_type() != BytecodeType::End {
    let val = frame.decoder.decode_val(&frame.registers);

    if reg_i < new_frame.registers.len() {
      // TODO: We should also stop writing into registers when hitting the
      // parameter count. This won't matter for correctly constructed
      // bytecode but hand-written assembly/bytecode may violate
      // optimization assumptions.
      new_frame.registers[reg_i] = val;
      reg_i += 1;
    }
  }

  frame.decoder.decode_type(); // End (TODO: assert)
}
