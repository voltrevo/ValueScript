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
  pub fn apply_unary_op(
    &mut self,
    op: fn(input: Val) -> Val,
  ) {
    let input = self.decoder.decode_val(&self.registers);

    let register_index = self.decoder.decode_register_index();

    if register_index.is_some() {
      self.registers[register_index.unwrap()] = op(input);
    }
  }

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
  pub fn run(&mut self, bytecode: &Rc<Vec<u8>>, params: &[String]) -> Val {
    let mut bd = BytecodeDecoder {
      data: bytecode.clone(),
      pos: 0,
    };

    let main_fn = bd.decode_val(&Vec::new());
    let mut frame = main_fn.make_frame().expect("bytecode does start with function");

    let mut reg_i = frame.param_start;
    let mut param_i = 0;

    while reg_i < frame.param_end && param_i < params.len() {
      frame.registers[reg_i] = Val::String(Rc::new(params[param_i].clone()));

      reg_i += 1;
      param_i += 1;
    }

    self.stack.push(frame);

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
      OpEq => frame.apply_binary_op(operations::op_eq),
      OpNe => frame.apply_binary_op(operations::op_ne),
      OpTripleEq => frame.apply_binary_op(operations::op_triple_eq),
      OpTripleNe => frame.apply_binary_op(operations::op_triple_ne),
      OpAnd => frame.apply_binary_op(operations::op_and),
      OpOr => frame.apply_binary_op(operations::op_or),

      OpNot => frame.apply_unary_op(operations::op_not),

      OpLess => frame.apply_binary_op(operations::op_less),
      OpLessEq => frame.apply_binary_op(operations::op_less_eq),
      OpGreater => frame.apply_binary_op(operations::op_greater),
      OpGreaterEq => frame.apply_binary_op(operations::op_greater_eq),
      OpNullishCoalesce => frame.apply_binary_op(operations::op_nullish_coalesce),
      OpOptionalChain => frame.apply_binary_op(operations::op_optional_chain),
      OpBitAnd => frame.apply_binary_op(operations::op_bit_and),
      OpBitOr => frame.apply_binary_op(operations::op_bit_or),

      OpBitNot => frame.apply_unary_op(operations::op_bit_not),

      OpBitXor => frame.apply_binary_op(operations::op_bit_xor),
      OpLeftShift => frame.apply_binary_op(operations::op_left_shift),
      OpRightShift => frame.apply_binary_op(operations::op_right_shift),
      OpRightShiftUnsigned => frame.apply_binary_op(operations::op_right_shift_unsigned),

      TypeOf => frame.apply_unary_op(operations::op_typeof),

      InstanceOf => frame.apply_binary_op(operations::op_instance_of),
      In => frame.apply_binary_op(operations::op_in),

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

        let params_array = params.as_array_data();

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

      Sub => frame.apply_binary_op(operations::op_sub),

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

      UnaryPlus => frame.apply_unary_op(operations::op_unary_plus),
      UnaryMinus => frame.apply_unary_op(operations::op_unary_minus),
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
