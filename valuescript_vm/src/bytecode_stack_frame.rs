use std::mem::take;

use valuescript_common::InstructionByte;

use crate::builtins::internal_error_builtin::ToInternalError;
use crate::builtins::type_error_builtin::ToTypeError;
use crate::bytecode_decoder::BytecodeDecoder;
use crate::bytecode_decoder::BytecodeType;
use crate::cat_stack_frame::CatStackFrame;
use crate::jsx_element::JsxElement;
use crate::native_function::ThisWrapper;
use crate::operations;
use crate::operations::op_delete;
use crate::stack_frame::FrameStepOk;
use crate::stack_frame::FrameStepResult;
use crate::stack_frame::{CallResult, StackFrame, StackFrameTrait};
use crate::vs_object::VsObject;
use crate::vs_value::ToDynamicVal;
use crate::vs_value::ToVal;
use crate::vs_value::{LoadFunctionResult, Val, ValTrait};

#[derive(Clone)]
pub struct BytecodeStackFrame {
  pub decoder: BytecodeDecoder,
  pub registers: Vec<Val>,
  pub const_this: bool,
  pub param_start: usize,
  pub param_end: usize,
  pub this_target: Option<usize>,
  pub return_target: Option<usize>,
  pub catch_setting: Option<CatchSetting>,
}

#[derive(Clone)]
pub struct CatchSetting {
  pub pos: usize,
  pub register: Option<usize>,
}

impl BytecodeStackFrame {
  pub fn apply_unary_op(&mut self, op: fn(input: &Val) -> Result<Val, Val>) -> Result<(), Val> {
    let input = self.decoder.decode_val(&mut self.registers);

    if let Some(register_index) = self.decoder.decode_register_index() {
      self.registers[register_index] = op(&input)?;
    }

    Ok(())
  }

  pub fn apply_binary_op(
    &mut self,
    op: fn(left: &Val, right: &Val) -> Result<Val, Val>,
  ) -> Result<(), Val> {
    let left = self.decoder.decode_val(&mut self.registers);
    let right = self.decoder.decode_val(&mut self.registers);

    if let Some(register_index) = self.decoder.decode_register_index() {
      self.registers[register_index] = op(&left, &right)?;
    }

    Ok(())
  }

  pub fn transfer_parameters(&mut self, new_frame: &mut StackFrame) {
    let bytecode_type = self.decoder.peek_type();

    if bytecode_type == BytecodeType::Array {
      self.decoder.decode_type();

      while self.decoder.peek_type() != BytecodeType::End {
        let p = self.decoder.decode_val(&mut self.registers);
        new_frame.write_param(p);
      }

      self.decoder.decode_type(); // End (TODO: assert)

      return;
    }

    let params = self.decoder.decode_val(&mut self.registers);

    match params {
      Val::Array(array_data) => {
        for param in &array_data.elements {
          new_frame.write_param(param.clone())
        }
      }
      _ => panic!("Unexpected non-array params"),
    }
  }

  pub fn decode_parameters(&mut self) -> Vec<Val> {
    let mut res = Vec::<Val>::new();

    let bytecode_type = self.decoder.peek_type();

    if bytecode_type == BytecodeType::Array {
      self.decoder.decode_type();

      while self.decoder.peek_type() != BytecodeType::End {
        res.push(self.decoder.decode_val(&mut self.registers));
      }

      self.decoder.decode_type(); // End (TODO: assert)

      return res;
    }

    let params = self.decoder.decode_val(&mut self.registers);

    match params {
      Val::Array(array_data) => array_data.elements.clone(),
      _ => panic!("Unexpected non-array params"),
    }
  }
}

impl StackFrameTrait for BytecodeStackFrame {
  fn write_this(&mut self, const_: bool, this: Val) -> Result<(), Val> {
    self.registers[1] = this;
    self.const_this = const_;
    Ok(())
  }

  fn write_param(&mut self, param: Val) {
    if self.param_start < self.param_end {
      self.registers[self.param_start] = param;
      self.param_start += 1;
    }
  }

  fn step(&mut self) -> FrameStepResult {
    use InstructionByte::*;

    let instruction_byte = self.decoder.decode_instruction();

    match instruction_byte {
      End => {
        return Ok(FrameStepOk::Pop(CallResult {
          return_: take(&mut self.registers[0]),
          this: take(&mut self.registers[1]),
        }));
      }

      Mov => {
        let val = self.decoder.decode_val(&mut self.registers);

        if let Some(register_index) = self.decoder.decode_register_index() {
          self.registers[register_index] = val;
        }
      }

      OpInc => {
        let register_index = self.decoder.decode_register_index().unwrap();
        let val = &mut self.registers[register_index];

        match val {
          Val::Number(n) => *n += 1.0,
          Val::BigInt(bi) => *bi += 1,
          _ => *val = operations::op_plus(val, &1.0.to_val())?,
        };
      }

      OpDec => {
        let register_index = self.decoder.decode_register_index().unwrap();
        let val = &mut self.registers[register_index];

        match val {
          Val::Number(n) => *n -= 1.0,
          Val::BigInt(bi) => *bi -= 1,
          _ => *val = operations::op_minus(val, &1.0.to_val())?,
        };
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

      OpNot => self.apply_unary_op(operations::op_not)?,

      OpLess => self.apply_binary_op(operations::op_less)?,
      OpLessEq => self.apply_binary_op(operations::op_less_eq)?,
      OpGreater => self.apply_binary_op(operations::op_greater)?,
      OpGreaterEq => self.apply_binary_op(operations::op_greater_eq)?,
      OpNullishCoalesce => self.apply_binary_op(operations::op_nullish_coalesce)?,
      OpOptionalChain => {
        let mut left = self.decoder.decode_val(&mut self.registers);
        let right = self.decoder.decode_val(&mut self.registers);

        if let Some(register_index) = self.decoder.decode_register_index() {
          self.registers[register_index] = operations::op_optional_chain(&mut left, &right)?;
        }
      }
      OpBitAnd => self.apply_binary_op(operations::op_bit_and)?,
      OpBitOr => self.apply_binary_op(operations::op_bit_or)?,

      OpBitNot => self.apply_unary_op(operations::op_bit_not)?,

      OpBitXor => self.apply_binary_op(operations::op_bit_xor)?,
      OpLeftShift => self.apply_binary_op(operations::op_left_shift)?,
      OpRightShift => self.apply_binary_op(operations::op_right_shift)?,
      OpRightShiftUnsigned => self.apply_binary_op(operations::op_right_shift_unsigned)?,

      TypeOf => self.apply_unary_op(operations::op_typeof)?,

      InstanceOf => self.apply_binary_op(operations::op_instance_of)?,
      In => self.apply_binary_op(operations::op_in)?,

      Call => {
        let fn_ = self.decoder.decode_val(&mut self.registers);

        match fn_.load_function() {
          LoadFunctionResult::NotAFunction => {
            return Err("fn_ is not a function".to_type_error());
          }
          LoadFunctionResult::StackFrame(mut new_frame) => {
            self.transfer_parameters(&mut new_frame);

            self.return_target = self.decoder.decode_register_index();
            self.this_target = None;

            return Ok(FrameStepOk::Push(new_frame));
          }
          LoadFunctionResult::NativeFunction(native_fn) => {
            let res = native_fn(
              ThisWrapper::new(true, &mut Val::Undefined),
              self.decode_parameters(),
            )?;

            if let Some(return_target) = self.decoder.decode_register_index() {
              self.registers[return_target] = res;
            };
          }
        };
      }

      Apply | ConstApply => {
        let fn_ = self.decoder.decode_val(&mut self.registers);

        match fn_.load_function() {
          LoadFunctionResult::NotAFunction => {
            return Err("fn_ is not a function".to_type_error());
          }
          LoadFunctionResult::StackFrame(mut new_frame) => {
            let this_target = self.decoder.decode_register_index();
            self.this_target = this_target;

            if this_target.is_some() {
              new_frame.write_this(
                instruction_byte == ConstApply,
                self.registers[this_target.unwrap()].clone(),
              )?;
            }

            self.transfer_parameters(&mut new_frame);

            self.return_target = self.decoder.decode_register_index();

            return Ok(FrameStepOk::Push(new_frame));
          }
          LoadFunctionResult::NativeFunction(_native_fn) => {
            return Err("TODO: apply native functions".to_internal_error());
          }
        }
      }

      Bind => {
        let fn_val = self.decoder.decode_val(&mut self.registers);
        let params = self.decoder.decode_val(&mut self.registers);
        let register_index = self.decoder.decode_register_index();

        let params_array = match params.as_array_data() {
          Some(params_array) => params_array,

          // Not sure this needs to be an exception in future since compiled
          // code should never violate this
          None => return Err("bind params should always be array".to_internal_error()),
        };

        let bound_fn = match fn_val.bind(params_array.elements.clone()) {
          Some(bound_fn) => bound_fn,

          // Not sure this needs to be an exception in future since compiled
          // code should never violate this
          None => return Err("fn parameter of bind should always be bindable".to_internal_error()),
        };

        if let Some(register_index) = register_index {
          self.registers[register_index] = bound_fn;
        }
      }

      Sub => {
        let mut left = self.decoder.decode_val(&mut self.registers);
        let right = self.decoder.decode_val(&mut self.registers);

        if let Some(register_index) = self.decoder.decode_register_index() {
          self.registers[register_index] = operations::op_sub(&mut left, &right)?;
        }
      }

      SubMov => {
        // TODO: Ideally we would use a reference for the subscript (decode_vallish), but that would
        // be an immutable borrow and it conflicts with the mutable borrow for the target. In
        // theory, this should still be possible because we only need a mutable borrow to an
        // element, not the vec itself. vec.get_many_mut has been considered, but it's not yet
        // stable.
        let subscript = self.decoder.decode_val(&mut self.registers);

        let value = self.decoder.decode_val(&mut self.registers);

        let target_index = self.decoder.decode_register_index().unwrap();

        operations::op_submov(&mut self.registers[target_index], &subscript, value)?;
      }

      ConstSubCall => {
        let const_call = true;

        let mut obj = self.decoder.decode_val(&mut self.registers);
        let subscript = self.decoder.decode_val(&mut self.registers);
        let fn_ = obj.sub(&subscript)?;

        match fn_.load_function() {
          LoadFunctionResult::NotAFunction => {
            return Err("fn_ is not a function".to_type_error());
          }
          LoadFunctionResult::StackFrame(mut new_frame) => {
            self.transfer_parameters(&mut new_frame);

            new_frame.write_this(const_call, obj)?;

            self.return_target = self.decoder.decode_register_index();
            self.this_target = None;

            return Ok(FrameStepOk::Push(new_frame));
          }
          LoadFunctionResult::NativeFunction(native_fn) => {
            let params = self.decode_parameters();

            let res = native_fn(ThisWrapper::new(true, &mut obj), params)?;

            if let Some(return_target) = self.decoder.decode_register_index() {
              self.registers[return_target] = res;
            };
          }
        };
      }

      SubCall | ThisSubCall => {
        let const_call = instruction_byte == InstructionByte::ConstSubCall
          || (instruction_byte == InstructionByte::ThisSubCall && self.const_this);

        let obj_i = self.decoder.decode_register_index().unwrap();
        let subscript = self.decoder.decode_val(&mut self.registers);
        let fn_ = self.registers[obj_i].sub(&subscript)?;

        match fn_.load_function() {
          LoadFunctionResult::NotAFunction => {
            return Err("fn_ is not a function".to_type_error());
          }
          LoadFunctionResult::StackFrame(mut new_frame) => {
            self.transfer_parameters(&mut new_frame);

            new_frame.write_this(const_call, take(&mut self.registers[obj_i]))?;

            self.return_target = self.decoder.decode_register_index();
            self.this_target = Some(obj_i);

            return Ok(FrameStepOk::Push(new_frame));
          }
          LoadFunctionResult::NativeFunction(native_fn) => {
            let params = self.decode_parameters();

            let res = native_fn(
              ThisWrapper::new(const_call, self.registers.get_mut(obj_i).unwrap()),
              params,
            )?;

            if let Some(return_target) = self.decoder.decode_register_index() {
              self.registers[return_target] = res;
            };
          }
        };
      }

      Jmp => {
        let dst = self.decoder.decode_pos();
        self.decoder.pos = dst;
      }

      JmpIf => {
        let cond = self.decoder.decode_val(&mut self.registers);
        let dst = self.decoder.decode_pos();

        if cond.is_truthy() {
          self.decoder.pos = dst;
        }
      }

      JmpIfNot => {
        let cond = self.decoder.decode_val(&mut self.registers);
        let dst = self.decoder.decode_pos();

        if !cond.is_truthy() {
          self.decoder.pos = dst;
        }
      }

      UnaryPlus => self.apply_unary_op(operations::op_unary_plus)?,
      UnaryMinus => self.apply_unary_op(operations::op_unary_minus)?,

      New => {
        // TODO: new Array

        let class = match self.decoder.decode_val(&mut self.registers).as_class_data() {
          Some(class) => class,
          None => {
            return Err("value is not a constructor".to_type_error());
          }
        };

        let mut instance = VsObject {
          string_map: Default::default(),
          symbol_map: Default::default(),
          prototype: class.prototype.clone(),
        }
        .to_val();

        match class.constructor {
          Val::Void => {
            // Ignore parameters
            self.decoder.decode_val(&mut self.registers);
            let target_register = self.decoder.decode_register_index();

            match target_register {
              None => {}
              Some(tr) => self.registers[tr] = instance,
            };
          }
          _ => match class.constructor.load_function() {
            LoadFunctionResult::NotAFunction => {
              return Err("fn_ is not a function".to_type_error());
            }
            LoadFunctionResult::StackFrame(mut new_frame) => {
              self.transfer_parameters(&mut new_frame);
              new_frame.write_this(false, instance)?;

              self.return_target = None;
              self.this_target = self.decoder.decode_register_index();

              return Ok(FrameStepOk::Push(new_frame));
            }
            LoadFunctionResult::NativeFunction(native_fn) => {
              native_fn(
                ThisWrapper::new(false, &mut instance),
                self.decode_parameters(),
              )?;

              if let Some(target) = self.decoder.decode_register_index() {
                self.registers[target] = instance;
              };
            }
          },
        };
      }

      Throw => {
        return match self.decoder.peek_type() {
          BytecodeType::TakeRegister => {
            self.decoder.decode_type();

            // Avoid the void->undefined conversion here
            let error = take(&mut self.registers[self.decoder.decode_register_index().unwrap()]);

            match error {
              Val::Void => Ok(FrameStepOk::Continue),
              _ => Err(error),
            }
          }
          BytecodeType::Register => {
            self.decoder.decode_type();

            // Avoid the void->undefined conversion here
            let error = self.registers[self.decoder.decode_register_index().unwrap()].clone();

            match error {
              Val::Void => Ok(FrameStepOk::Continue),
              _ => Err(error),
            }
          }
          _ => Err(self.decoder.decode_val(&mut self.registers)),
        };
      }

      Import | ImportStar => {
        return Err("TODO: Dynamic imports".to_internal_error());
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

      RequireMutableThis => {
        if self.const_this {
          return Err("Cannot mutate this because it is const".to_type_error());
        }
      }

      Next => {
        let iter_i = match self.decoder.decode_register_index() {
          Some(i) => i,
          None => return Err("The ignore register is not iterable".to_internal_error()),
        };

        let res_i = self.decoder.decode_register_index();

        let next_fn = self.registers[iter_i].sub(&"next".to_val())?;

        match next_fn.load_function() {
          LoadFunctionResult::NotAFunction => {
            return Err(".next() is not a function".to_type_error())
          }
          LoadFunctionResult::NativeFunction(fn_) => {
            let res = fn_(ThisWrapper::new(false, &mut self.registers[iter_i]), vec![])?;

            if let Some(res_i) = res_i {
              self.registers[res_i] = res;
            }
          }
          LoadFunctionResult::StackFrame(mut new_frame) => {
            new_frame.write_this(false, take(&mut self.registers[iter_i]))?;

            self.return_target = res_i;
            self.this_target = Some(iter_i);

            return Ok(FrameStepOk::Push(new_frame));
          }
        };
      }

      UnpackIterRes => {
        let iter_res_i = match self.decoder.decode_register_index() {
          Some(i) => i,
          None => return Err("Can't unpack the ignore register".to_internal_error()),
        };

        let iter_res = take(&mut self.registers[iter_res_i]);

        if let Some(value_i) = self.decoder.decode_register_index() {
          self.registers[value_i] = iter_res.sub(&"value".to_val())?;
        }

        if let Some(done_i) = self.decoder.decode_register_index() {
          self.registers[done_i] = iter_res.sub(&"done".to_val())?;
        }
      }

      Cat => {
        let cat_frame = match self.decoder.peek_type() {
          BytecodeType::Array => {
            self.decoder.decode_type();
            CatStackFrame::from_vec_val(self.decoder.decode_vec_val(&mut self.registers))
          }
          _ => match self.decoder.decode_val(&mut self.registers) {
            Val::Array(array) => CatStackFrame::from_vec_val(array.elements.clone()),
            _ => {
              return Err(
                "TODO: cat instruction on non-array (usually type error)".to_internal_error(),
              )
            }
          },
        };

        self.this_target = None;
        self.return_target = self.decoder.decode_register_index();

        return Ok(FrameStepOk::Push(Box::new(cat_frame)));
      }

      Yield => {
        let val = self.decoder.decode_val(&mut self.registers);
        self.decoder.decode_register_index(); // TODO: Use this

        return Ok(FrameStepOk::Yield(val));
      }

      YieldStar => {
        let val = self.decoder.decode_val(&mut self.registers);
        self.decoder.decode_register_index(); // TODO: Use this

        return Ok(FrameStepOk::YieldStar(val));
      }

      Delete => {
        let obj_i = self
          .decoder
          .decode_register_index()
          .expect("Need reg index for delete");

        let mut obj = take(&mut self.registers[obj_i]);

        let prop = self.decoder.decode_val(&mut self.registers);

        op_delete(&mut obj, &prop)?;

        if let Some(i) = self.decoder.decode_register_index() {
          self.registers[i] = Val::Bool(true);
        }

        self.registers[obj_i] = obj;

        return Ok(FrameStepOk::Continue);
      }

      Jsx => {
        let tag = self.decoder.decode_val(&mut self.registers);
        let attrs_val = self.decoder.decode_val(&mut self.registers);
        let children_val = self.decoder.decode_val(&mut self.registers);

        let mut attrs = Vec::<(String, Val)>::new();

        match attrs_val {
          Val::Array(array) => {
            for attr in &array.elements {
              match attr {
                Val::Array(array) => {
                  if array.elements.len() != 2 {
                    return Err("Unexpected non-pair attribute".to_type_error());
                  }

                  let name = match &array.elements[0] {
                    Val::String(str) => str.to_string(),
                    _ => return Err("Unexpected non-string attribute name".to_type_error()),
                  };

                  let value = array.elements[1].clone();

                  attrs.push((name, value));
                }
                _ => return Err("Unexpected non-array attrs".to_type_error()),
              }
            }
          }
          _ => return Err("Unexpected non-array attrs".to_type_error()),
        }

        let children = match children_val {
          Val::Array(array) => array.elements.clone(),
          _ => return Err("Unexpected non-array children".to_type_error()),
        };

        let res = JsxElement {
          tag: match tag {
            Val::Void => None,
            Val::String(str) => Some(str.to_string()),
            _ => return Err("Unexpected non-string tag".to_type_error()),
          },
          attrs,
          children,
        }
        .to_dynamic_val();

        if let Some(i) = self.decoder.decode_register_index() {
          self.registers[i] = res;
        }

        return Ok(FrameStepOk::Continue);
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

  fn catch_exception(&mut self, exception: &mut Val) {
    if let Some(catch_setting) = &self.catch_setting {
      let exception = take(exception);

      if let Some(r) = catch_setting.register {
        self.registers[r] = exception;
      }

      self.decoder.pos = catch_setting.pos;
      self.catch_setting = None;
    }
  }

  fn clone_to_stack_frame(&self) -> StackFrame {
    Box::new(self.clone())
  }
}
