use std::rc::Rc;

use super::vs_value::{Val, ValTrait, LoadFunctionResult};
use super::operations;
use super::bytecode_decoder::BytecodeDecoder;
use super::bytecode_decoder::BytecodeType;
use super::instruction::Instruction;

pub struct VirtualMachine {
  pub frame: StackFrame,
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

enum ThisArg {
  Register(usize),
  Val(Val),
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

    let mut frame = match main_fn.load_function() {
      LoadFunctionResult::StackFrame(f) => f,
      _ => std::panic!("bytecode does start with function"),
    };

    let mut reg_i = frame.param_start;
    let mut param_i = 0;

    while reg_i < frame.param_end && param_i < params.len() {
      frame.registers[reg_i] = Val::String(Rc::new(params[param_i].clone()));

      reg_i += 1;
      param_i += 1;
    }

    self.push(frame);

    while self.stack.len() > 0 {
      self.step();
    }

    return self.frame.registers[0].clone();
  }

  pub fn new() -> VirtualMachine {
    let mut registers: Vec<Val> = Vec::with_capacity(2);
    registers.push(Val::Undefined);
    registers.push(Val::Undefined);

    return VirtualMachine {
      frame: StackFrame {
        decoder: BytecodeDecoder {
          data: Rc::new(Vec::new()),
          pos: 0,
        },
        registers: registers,
        param_start: 2,
        param_end: 2,
        return_target: Some(0),
        this_target: Some(1),
      },
      stack: Default::default(),
    };
  }

  pub fn step(&mut self) {
    use Instruction::*;

    match self.frame.decoder.decode_instruction() {
      End => {
        self.pop();
      },

      Mov => {
        let val = self.frame.decoder.decode_val(&self.frame.registers);
        let register_index = self.frame.decoder.decode_register_index();

        if register_index.is_some() {
          self.frame.registers[register_index.unwrap()] = val;
        }
      },

      OpInc => {
        let register_index = self.frame.decoder.decode_register_index().unwrap();
        let mut val = self.frame.registers[register_index].clone();
        val = operations::op_plus(val, Val::Number(1_f64));
        self.frame.registers[register_index] = val;
      },

      OpDec => {
        let register_index = self.frame.decoder.decode_register_index().unwrap();
        let mut val = self.frame.registers[register_index].clone();
        val = operations::op_minus(val, Val::Number(1_f64));
        self.frame.registers[register_index] = val;
      },

      OpPlus => self.frame.apply_binary_op(operations::op_plus),
      OpMinus => self.frame.apply_binary_op(operations::op_minus),
      OpMul => self.frame.apply_binary_op(operations::op_mul),
      OpDiv => self.frame.apply_binary_op(operations::op_div),
      OpMod => self.frame.apply_binary_op(operations::op_mod),
      OpExp => self.frame.apply_binary_op(operations::op_exp),
      OpEq => self.frame.apply_binary_op(operations::op_eq),
      OpNe => self.frame.apply_binary_op(operations::op_ne),
      OpTripleEq => self.frame.apply_binary_op(operations::op_triple_eq),
      OpTripleNe => self.frame.apply_binary_op(operations::op_triple_ne),
      OpAnd => self.frame.apply_binary_op(operations::op_and),
      OpOr => self.frame.apply_binary_op(operations::op_or),

      OpNot => self.frame.apply_unary_op(operations::op_not),

      OpLess => self.frame.apply_binary_op(operations::op_less),
      OpLessEq => self.frame.apply_binary_op(operations::op_less_eq),
      OpGreater => self.frame.apply_binary_op(operations::op_greater),
      OpGreaterEq => self.frame.apply_binary_op(operations::op_greater_eq),
      OpNullishCoalesce => self.frame.apply_binary_op(operations::op_nullish_coalesce),
      OpOptionalChain => self.frame.apply_binary_op(operations::op_optional_chain),
      OpBitAnd => self.frame.apply_binary_op(operations::op_bit_and),
      OpBitOr => self.frame.apply_binary_op(operations::op_bit_or),

      OpBitNot => self.frame.apply_unary_op(operations::op_bit_not),

      OpBitXor => self.frame.apply_binary_op(operations::op_bit_xor),
      OpLeftShift => self.frame.apply_binary_op(operations::op_left_shift),
      OpRightShift => self.frame.apply_binary_op(operations::op_right_shift),
      OpRightShiftUnsigned => self.frame.apply_binary_op(operations::op_right_shift_unsigned),

      TypeOf => self.frame.apply_unary_op(operations::op_typeof),

      InstanceOf => self.frame.apply_binary_op(operations::op_instance_of),
      In => self.frame.apply_binary_op(operations::op_in),

      Call => {
        let fn_ = self.frame.decoder.decode_val(&self.frame.registers);

        match fn_.load_function() {
          LoadFunctionResult::NotAFunction => 
            std::panic!("Not implemented: throw exception (fn_ is not a function)")
          ,
          LoadFunctionResult::StackFrame(mut new_frame) => {
            transfer_parameters(&mut self.frame, &mut new_frame);
    
            self.frame.return_target = self.frame.decoder.decode_register_index();
            self.frame.this_target = None;
    
            self.push(new_frame);
          },
          LoadFunctionResult::NativeFunction(native_fn) => {
            let res = native_fn(
              &mut Val::Undefined,
              get_parameters(&mut self.frame),
            );
            
            match self.frame.decoder.decode_register_index() {
              Some(return_target) => {
                self.frame.registers[return_target] = res;
              },
              None => {},
            };
          },
        };
      }

      Apply => {
        let fn_ = self.frame.decoder.decode_val(&self.frame.registers);

        match fn_.load_function() {
          LoadFunctionResult::NotAFunction => 
            std::panic!("Not implemented: throw exception (fn_ is not a function)")
          ,
          LoadFunctionResult::StackFrame(mut new_frame) => {
            if self.frame.decoder.peek_type() == BytecodeType::Register {
              self.frame.decoder.decode_type();
              let this_target = self.frame.decoder.decode_register_index();
              self.frame.this_target = this_target;
    
              if this_target.is_some() {
                new_frame.registers[1] = self.frame.registers[this_target.unwrap()].clone();
              }
            } else {
              self.frame.this_target = None;
              new_frame.registers[1] = self.frame.decoder.decode_val(&self.frame.registers);
            }
    
            transfer_parameters(&mut self.frame, &mut new_frame);
    
            self.frame.return_target = self.frame.decoder.decode_register_index();
    
            self.push(new_frame);
          },
          LoadFunctionResult::NativeFunction(native_fn) => {
            std::panic!("Not implemented");
          },
        }
      }

      Bind => {
        let fn_val = self.frame.decoder.decode_val(&self.frame.registers);
        let params = self.frame.decoder.decode_val(&self.frame.registers);
        let register_index = self.frame.decoder.decode_register_index();

        let params_array = params.as_array_data();

        if params_array.is_none() {
          // Not sure this needs to be an exception in future since compiled
          // code should never violate this
          std::panic!("bind params should always be array")
        }

        let bound_fn = fn_val.bind((*params_array.unwrap()).elements.clone());

        if bound_fn.is_none() {
          // Not sure this needs to be an exception in future since compiled
          // code should never violate this
          std::panic!("fn parameter of bind should always be bindable");
        }

        if register_index.is_some() {
          self.frame.registers[register_index.unwrap()] = bound_fn.unwrap();
        }
      },

      Sub => self.frame.apply_binary_op(operations::op_sub),

      SubMov => {
        let subscript = self.frame.decoder.decode_val(&self.frame.registers);
        let value = self.frame.decoder.decode_val(&self.frame.registers);
    
        let register_index = self.frame.decoder.decode_register_index().unwrap();
        let mut target = self.frame.registers[register_index].clone(); // TODO: Lift

        operations::op_submov(&mut target, subscript, value);
        self.frame.registers[register_index] = target;
      },

      SubCall => {
        let mut obj = match self.frame.decoder.peek_type() {
          BytecodeType::Register => {
            self.frame.decoder.decode_type();

            ThisArg::Register(
              self.frame.decoder.decode_register_index().unwrap()
            )
          },
          _ => ThisArg::Val(self.frame.decoder.decode_val(&self.frame.registers)),
        };

        let subscript = self.frame.decoder.decode_val(&self.frame.registers);

        let fn_ = operations::op_sub(
          match &obj {
            ThisArg::Register(reg_i) => self.frame.registers[reg_i.clone()].clone(),
            ThisArg::Val(val) => val.clone(),
          },
          subscript,
        );

        match fn_.load_function() {
          LoadFunctionResult::NotAFunction => 
            std::panic!("Not implemented: throw exception (fn_ is not a function)")
          ,
          LoadFunctionResult::StackFrame(mut new_frame) => {
            transfer_parameters(&mut self.frame, &mut new_frame);

            new_frame.registers[1] = match &obj {
              ThisArg::Register(reg_i) => self.frame.registers[reg_i.clone()].clone(),
              ThisArg::Val(val) => val.clone(),
            };
    
            self.frame.return_target = self.frame.decoder.decode_register_index();

            self.frame.this_target = match obj {
              ThisArg::Register(reg_i) => Some(reg_i),
              ThisArg::Val(_) => None,
            };
    
            self.push(new_frame);
          },
          LoadFunctionResult::NativeFunction(native_fn) => {
            let params = get_parameters(&mut self.frame);

            let res = match &mut obj {
              ThisArg::Register(reg_i) => {
                native_fn(
                  self.frame.registers.get_mut(reg_i.clone()).unwrap(),
                  params,
                )
              },
              ThisArg::Val(val) => {
                native_fn(
                  val,
                  params,
                )
              },
            };
            
            match self.frame.decoder.decode_register_index() {
              Some(return_target) => {
                self.frame.registers[return_target] = res;
              },
              None => {},
            };
          },
        };
      },

      Jmp => {
        let dst = self.frame.decoder.decode_pos();
        self.frame.decoder.pos = dst;
      }

      JmpIf => {
        let cond = self.frame.decoder.decode_val(&self.frame.registers);
        let dst = self.frame.decoder.decode_pos();

        if cond.is_truthy() {
          self.frame.decoder.pos = dst;
        }
      }

      UnaryPlus => self.frame.apply_unary_op(operations::op_unary_plus),
      UnaryMinus => self.frame.apply_unary_op(operations::op_unary_minus),

      New => std::panic!("Not implemented"),
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

    for return_target in self.frame.return_target {
      self.frame.registers[return_target] = old_frame.registers[0].clone();
    }

    for this_target in self.frame.this_target {
      self.frame.registers[this_target] = old_frame.registers[1].clone();
    }
  }
}

fn get_parameters(
  frame: &mut StackFrame,
) -> Vec<Val> {
  let mut res = Vec::<Val>::new();

  let bytecode_type = frame.decoder.decode_type();

  if bytecode_type != BytecodeType::Array {
    std::panic!("Not implemented: call instruction not using inline array");
  }

  while frame.decoder.peek_type() != BytecodeType::End {
    res.push(frame.decoder.decode_val(&frame.registers));
  }

  frame.decoder.decode_type(); // End (TODO: assert)

  return res;
}

fn transfer_parameters(
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
