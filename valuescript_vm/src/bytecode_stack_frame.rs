use std::rc::Rc;

use valuescript_common::InstructionByte;

use crate::builtins::type_error_builtin::to_type_error;
use crate::bytecode_decoder::BytecodeDecoder;
use crate::bytecode_decoder::BytecodeType;
use crate::operations;
use crate::stack_frame::FrameStepOk;
use crate::stack_frame::FrameStepResult;
use crate::stack_frame::{CallResult, StackFrame, StackFrameTrait};
use crate::type_error;
use crate::vs_object::VsObject;
use crate::vs_value::{LoadFunctionResult, Val, ValTrait};

pub struct BytecodeStackFrame {
  pub decoder: BytecodeDecoder,
  pub registers: Vec<Val>,
  pub param_start: usize,
  pub param_end: usize,
  pub this_target: Option<usize>,
  pub return_target: Option<usize>,
  pub catch_setting: Option<CatchSetting>,
}

pub struct CatchSetting {
  pub pos: usize,
  pub register: Option<usize>,
}

impl BytecodeStackFrame {
  pub fn apply_unary_op(&mut self, op: fn(input: Val) -> Val) {
    let input = self.decoder.decode_val(&self.registers);

    let register_index = self.decoder.decode_register_index();

    if register_index.is_some() {
      self.registers[register_index.unwrap()] = op(input);
    }
  }

  pub fn apply_binary_op(
    &mut self,
    op: fn(left: Val, right: Val) -> Result<Val, Val>,
  ) -> Result<(), Val> {
    let left = self.decoder.decode_val(&self.registers);
    let right = self.decoder.decode_val(&self.registers);

    let register_index = self.decoder.decode_register_index();

    if register_index.is_some() {
      self.registers[register_index.unwrap()] = op(left, right)?;
    }

    Ok(())
  }

  pub fn transfer_parameters(&mut self, new_frame: &mut StackFrame) {
    let bytecode_type = self.decoder.decode_type();

    if bytecode_type != BytecodeType::Array {
      panic!("Not implemented: call instruction not using inline array");
    }

    while self.decoder.peek_type() != BytecodeType::End {
      let p = self.decoder.decode_val(&self.registers);
      new_frame.write_param(p);
    }

    self.decoder.decode_type(); // End (TODO: assert)
  }

  pub fn decode_parameters(&mut self) -> Vec<Val> {
    let mut res = Vec::<Val>::new();

    let bytecode_type = self.decoder.decode_type();

    if bytecode_type != BytecodeType::Array {
      panic!("Not implemented: call instruction not using inline array");
    }

    while self.decoder.peek_type() != BytecodeType::End {
      res.push(self.decoder.decode_val(&self.registers));
    }

    self.decoder.decode_type(); // End (TODO: assert)

    return res;
  }
}

impl StackFrameTrait for BytecodeStackFrame {
  fn write_this(&mut self, this: Val) {
    self.registers[1] = this;
  }

  fn write_param(&mut self, param: Val) {
    if self.param_start < self.param_end {
      self.registers[self.param_start] = param;
      self.param_start += 1;
    }
  }

  fn step(&mut self) -> FrameStepResult {
    use InstructionByte::*;

    match self.decoder.decode_instruction() {
      End => {
        return Ok(FrameStepOk::Pop(CallResult {
          return_: self.registers[0].clone(),
          this: self.registers[1].clone(),
        }));
      }

      Mov => {
        let val = self.decoder.decode_val(&self.registers);
        let register_index = self.decoder.decode_register_index();

        if register_index.is_some() {
          self.registers[register_index.unwrap()] = val;
        }
      }

      OpInc => {
        let register_index = self.decoder.decode_register_index().unwrap();
        let mut val = self.registers[register_index].clone();
        val = operations::op_plus(val, Val::Number(1_f64))?; // TODO: BigInt
        self.registers[register_index] = val;
      }

      OpDec => {
        let register_index = self.decoder.decode_register_index().unwrap();
        let mut val = self.registers[register_index].clone();
        val = operations::op_minus(val, Val::Number(1_f64))?; // TODO: BigInt
        self.registers[register_index] = val;
      }

      OpPlus => self.apply_binary_op(operations::op_plus)?,
      OpMinus => self.apply_binary_op(operations::op_minus)?,
      OpMul => self.apply_binary_op(operations::op_mul)?,
      OpDiv => self.apply_binary_op(operations::op_div)?,
      OpMod => self.apply_binary_op(operations::op_mod)?,
      OpExp => self.apply_binary_op(operations::op_exp)?,
      OpEq => self.apply_binary_op(operations::op_eq)?,
      OpNe => self.apply_binary_op(operations::op_ne)?,
      OpTripleEq => self.apply_binary_op(operations::op_triple_eq)?,
      OpTripleNe => self.apply_binary_op(operations::op_triple_ne)?,
      OpAnd => self.apply_binary_op(operations::op_and)?,
      OpOr => self.apply_binary_op(operations::op_or)?,

      OpNot => self.apply_unary_op(operations::op_not),

      OpLess => self.apply_binary_op(operations::op_less)?,
      OpLessEq => self.apply_binary_op(operations::op_less_eq)?,
      OpGreater => self.apply_binary_op(operations::op_greater)?,
      OpGreaterEq => self.apply_binary_op(operations::op_greater_eq)?,
      OpNullishCoalesce => self.apply_binary_op(operations::op_nullish_coalesce)?,
      OpOptionalChain => self.apply_binary_op(operations::op_optional_chain)?,
      OpBitAnd => self.apply_binary_op(operations::op_bit_and)?,
      OpBitOr => self.apply_binary_op(operations::op_bit_or)?,

      OpBitNot => self.apply_unary_op(operations::op_bit_not),

      OpBitXor => self.apply_binary_op(operations::op_bit_xor)?,
      OpLeftShift => self.apply_binary_op(operations::op_left_shift)?,
      OpRightShift => self.apply_binary_op(operations::op_right_shift)?,
      OpRightShiftUnsigned => self.apply_binary_op(operations::op_right_shift_unsigned)?,

      TypeOf => self.apply_unary_op(operations::op_typeof),

      InstanceOf => self.apply_binary_op(operations::op_instance_of)?,
      In => self.apply_binary_op(operations::op_in)?,

      Call => {
        let fn_ = self.decoder.decode_val(&self.registers);

        match fn_.load_function() {
          LoadFunctionResult::NotAFunction => {
            panic!("Not implemented: throw exception (fn_ is not a function)")
          }
          LoadFunctionResult::StackFrame(mut new_frame) => {
            self.transfer_parameters(&mut new_frame);

            self.return_target = self.decoder.decode_register_index();
            self.this_target = None;

            return Ok(FrameStepOk::Push(new_frame));
          }
          LoadFunctionResult::NativeFunction(native_fn) => {
            let res = native_fn(&mut Val::Undefined, self.decode_parameters())?;

            match self.decoder.decode_register_index() {
              Some(return_target) => {
                self.registers[return_target] = res;
              }
              None => {}
            };
          }
        };
      }

      Apply => {
        let fn_ = self.decoder.decode_val(&self.registers);

        match fn_.load_function() {
          LoadFunctionResult::NotAFunction => {
            panic!("Not implemented: throw exception (fn_ is not a function)")
          }
          LoadFunctionResult::StackFrame(mut new_frame) => {
            if self.decoder.peek_type() == BytecodeType::Register {
              self.decoder.decode_type();
              let this_target = self.decoder.decode_register_index();
              self.this_target = this_target;

              if this_target.is_some() {
                new_frame.write_this(self.registers[this_target.unwrap()].clone());
              }
            } else {
              self.this_target = None;
              new_frame.write_this(self.decoder.decode_val(&self.registers));
            }

            self.transfer_parameters(&mut new_frame);

            self.return_target = self.decoder.decode_register_index();

            return Ok(FrameStepOk::Push(new_frame));
          }
          LoadFunctionResult::NativeFunction(_native_fn) => {
            panic!("Not implemented");
          }
        }
      }

      Bind => {
        let fn_val = self.decoder.decode_val(&self.registers);
        let params = self.decoder.decode_val(&self.registers);
        let register_index = self.decoder.decode_register_index();

        let params_array = params.as_array_data();

        if params_array.is_none() {
          // Not sure this needs to be an exception in future since compiled
          // code should never violate this
          panic!("bind params should always be array")
        }

        let bound_fn = fn_val.bind((*params_array.unwrap()).elements.clone());

        if bound_fn.is_none() {
          // Not sure this needs to be an exception in future since compiled
          // code should never violate this
          panic!("fn parameter of bind should always be bindable");
        }

        if register_index.is_some() {
          self.registers[register_index.unwrap()] = bound_fn.unwrap();
        }
      }

      Sub => self.apply_binary_op(operations::op_sub)?,

      SubMov => {
        let subscript = self.decoder.decode_val(&self.registers);
        let value = self.decoder.decode_val(&self.registers);

        let register_index = self.decoder.decode_register_index().unwrap();
        let mut target = self.registers[register_index].clone(); // TODO: Lift

        operations::op_submov(&mut target, subscript, value)?;
        self.registers[register_index] = target;
      }

      SubCall => {
        let mut obj = match self.decoder.peek_type() {
          BytecodeType::Register => {
            self.decoder.decode_type();

            ThisArg::Register(self.decoder.decode_register_index().unwrap())
          }
          _ => ThisArg::Val(self.decoder.decode_val(&self.registers)),
        };

        let subscript = self.decoder.decode_val(&self.registers);

        let fn_ = operations::op_sub(
          match &obj {
            ThisArg::Register(reg_i) => self.registers[reg_i.clone()].clone(),
            ThisArg::Val(val) => val.clone(),
          },
          subscript,
        )?;

        match fn_.load_function() {
          LoadFunctionResult::NotAFunction => {
            panic!("Not implemented: throw exception (fn_ is not a function)")
          }
          LoadFunctionResult::StackFrame(mut new_frame) => {
            self.transfer_parameters(&mut new_frame);

            new_frame.write_this(match &obj {
              ThisArg::Register(reg_i) => self.registers[reg_i.clone()].clone(),
              ThisArg::Val(val) => val.clone(),
            });

            self.return_target = self.decoder.decode_register_index();

            self.this_target = match obj {
              ThisArg::Register(reg_i) => Some(reg_i),
              ThisArg::Val(_) => None,
            };

            return Ok(FrameStepOk::Push(new_frame));
          }
          LoadFunctionResult::NativeFunction(native_fn) => {
            let params = self.decode_parameters();

            let res = match &mut obj {
              ThisArg::Register(reg_i) => {
                native_fn(self.registers.get_mut(reg_i.clone()).unwrap(), params)?
              }
              ThisArg::Val(val) => native_fn(val, params)?,
            };

            match self.decoder.decode_register_index() {
              Some(return_target) => {
                self.registers[return_target] = res;
              }
              None => {}
            };
          }
        };
      }

      Jmp => {
        let dst = self.decoder.decode_pos();
        self.decoder.pos = dst;
      }

      JmpIf => {
        let cond = self.decoder.decode_val(&self.registers);
        let dst = self.decoder.decode_pos();

        if cond.is_truthy() {
          self.decoder.pos = dst;
        }
      }

      UnaryPlus => self.apply_unary_op(operations::op_unary_plus),
      UnaryMinus => self.apply_unary_op(operations::op_unary_minus),

      New => {
        // TODO: new Array

        let class = match self.decoder.decode_val(&self.registers).as_class_data() {
          Some(class) => class,
          None => {
            return type_error!("value is not a constructor");
          }
        };

        let mut instance = Val::Object(Rc::new(VsObject {
          string_map: Default::default(),
          prototype: Some(class.instance_prototype.clone()),
        }));

        match class.constructor {
          Val::Void => {
            // Ignore parameters
            self.decoder.decode_val(&self.registers);
            let target_register = self.decoder.decode_register_index();

            match target_register {
              None => {}
              Some(tr) => self.registers[tr] = instance,
            };
          }
          _ => match class.constructor.load_function() {
            LoadFunctionResult::NotAFunction => {
              panic!("Not implemented: throw exception (class.constructor is not a function)")
            }
            LoadFunctionResult::StackFrame(mut new_frame) => {
              self.transfer_parameters(&mut new_frame);
              new_frame.write_this(instance);

              self.return_target = None;
              self.this_target = self.decoder.decode_register_index();

              return Ok(FrameStepOk::Push(new_frame));
            }
            LoadFunctionResult::NativeFunction(native_fn) => {
              native_fn(&mut instance, self.decode_parameters())?;

              match self.decoder.decode_register_index() {
                Some(target) => {
                  self.registers[target] = instance;
                }
                None => {}
              };
            }
          },
        };
      }

      Throw => {
        let error = self.decoder.decode_val(&self.registers);
        return Err(error);
      }

      Import | ImportStar => {
        panic!("TODO: Dynamic imports")
      }

      SetCatch => {
        self.catch_setting = Some(CatchSetting {
          pos: self.decoder.decode_pos(),
          register: self.decoder.decode_register_index(),
        });
      }

      UnsetCatch => {
        self.catch_setting = None;
      }
    };

    Ok(FrameStepOk::Continue)
  }

  fn apply_call_result(&mut self, call_result: CallResult) {
    match self.this_target {
      None => {}
      Some(tt) => {
        self.registers[tt] = call_result.this;
      }
    };

    match self.return_target {
      None => {}
      Some(rt) => {
        self.registers[rt] = call_result.return_;
      }
    };
  }

  fn get_call_result(&mut self) -> CallResult {
    panic!("Not appropriate for BytecodeStackFrame")
  }

  fn catch_exception(&mut self, exception: Val) -> bool {
    if let Some(catch_setting) = &self.catch_setting {
      if let Some(r) = catch_setting.register {
        self.registers[r] = exception;
      }

      self.decoder.pos = catch_setting.pos;
      self.catch_setting = None;

      true
    } else {
      false
    }
  }
}

enum ThisArg {
  Register(usize),
  Val(Val),
}
