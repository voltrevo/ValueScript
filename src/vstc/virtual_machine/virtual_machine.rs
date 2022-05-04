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
  pub param_start: usize,
  pub param_end: usize,
  pub this_target: Option<usize>,
  pub return_target: Option<usize>,
}

impl StackFrame {
  pub fn apply_binary_op(
    &mut self,
    op: fn(left: Val, right: Val) -> Val,
  ) {
    let left = self.decoder.decode_val(&self.registers);
    let right = self.decoder.decode_val(&self.registers);

    let register_index = self.decoder.decode_register_index();

    if register_index.is_some() {
      self.registers[register_index.unwrap()] = op(left, right);
    }
  }
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
      param_start: 2,
      param_end: 2,
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
        val = operations::op_plus(val, Val::Number(1_f64));
        frame.registers[register_index] = val;
      },

      OpDec => {
        let register_index = frame.decoder.decode_register_index().unwrap();
        let mut val = frame.registers[register_index].clone();
        val = operations::op_minus(val, Val::Number(1_f64));
        frame.registers[register_index] = val;
      },

      OpPlus => frame.apply_binary_op(operations::op_plus),
      OpMinus => frame.apply_binary_op(operations::op_minus),
      OpMul => frame.apply_binary_op(operations::op_mul),
      OpDiv => frame.apply_binary_op(operations::op_div),
      OpMod => frame.apply_binary_op(operations::op_mod),
      OpExp => frame.apply_binary_op(operations::op_exp),

      OpEq => std::panic!("Instruction not implemented: OpEq"),

      OpNe => std::panic!("Instruction not implemented: OpNe"),

      OpTripleEq => std::panic!("Instruction not implemented: OpTripleEq"),

      OpTripleNe => frame.apply_binary_op(operations::op_triple_ne),
      OpAnd => frame.apply_binary_op(operations::op_and),
      OpOr => frame.apply_binary_op(operations::op_or),

      OpNot => std::panic!("Instruction not implemented: OpNot"),

      OpLess => frame.apply_binary_op(operations::op_less),

      OpLessEq => std::panic!("Instruction not implemented: OpLessEq"),

      OpGreater => std::panic!("Instruction not implemented: OpGreater"),

      OpGreaterEq => std::panic!("Instruction not implemented: OpGreaterEq"),

      OpNullishCoalesce => std::panic!("Instruction not implemented: OpNullishCoalesce"),

      OpOptionalChain => std::panic!("Instruction not implemented: OpOptionalChain"),

      OpBitAnd => std::panic!("Instruction not implemented: OpBitAnd"),

      OpBitOr => std::panic!("Instruction not implemented: OpBitOr"),

      OpBitNot => std::panic!("Instruction not implemented: OpBitNot"),

      OpBitXor => std::panic!("Instruction not implemented: OpBitXor"),

      OpLeftShift => std::panic!("Instruction not implemented: OpLeftShift"),

      OpRightShift => std::panic!("Instruction not implemented: OpRightShift"),

      OpRightShiftUnsigned => std::panic!("Instruction not implemented: OpRightShiftUnsigned"),

      TypeOf => std::panic!("Instruction not implemented: TypeOf"),

      InstanceOf => std::panic!("Instruction not implemented: InstanceOf"),

      In => std::panic!("Instruction not implemented: In"),

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

      Bind => {
        let fn_val = frame.decoder.decode_val(&frame.registers);
        let params = frame.decoder.decode_val(&frame.registers);
        let register_index = frame.decoder.decode_register_index();

        let params_array = params.as_array();

        if params_array.is_none() {
          // Not sure this needs to be an exception in future since compiled
          // code should never violate this
          std::panic!("bind params should always be array")
        }

        let bound_fn = fn_val.bind((*params_array.unwrap()).clone());

        if bound_fn.is_none() {
          // Not sure this needs to be an exception in future since compiled
          // code should never violate this
          std::panic!("fn parameter of bind should always be bindable");
        }

        if register_index.is_some() {
          frame.registers[register_index.unwrap()] = bound_fn.unwrap();
        }
      },

      Sub => std::panic!("Instruction not implemented: Sub"),

      SubMov => std::panic!("Instruction not implemented: SubMov"),

      SubCall => std::panic!("Instruction not implemented: SubCall"),

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

  let mut reg_i = new_frame.param_start;

  while frame.decoder.peek_type() != BytecodeType::End {
    let val = frame.decoder.decode_val(&frame.registers);

    if reg_i < new_frame.param_end {
      new_frame.registers[reg_i] = val;
      reg_i += 1;
    }
  }

  frame.decoder.decode_type(); // End (TODO: assert)
}
