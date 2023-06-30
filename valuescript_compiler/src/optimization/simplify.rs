use std::collections::{HashMap, HashSet};

use num_bigint::BigInt;
use valuescript_vm::{operations, vs_value::Val};

use crate::{
  asm::{DefinitionContent, FnLine, Function, Instruction, Module, Number, Register, Value},
  instruction::InstructionFieldMut,
  TryToVal,
};

use super::try_to_value::TryToValue;

pub fn simplify(module: &mut Module) {
  for defn in &mut module.definitions {
    match &mut defn.content {
      DefinitionContent::Function(fn_) => simplify_fn(FnState::default(), fn_),
      DefinitionContent::Class(_) => {}
      DefinitionContent::Value(_) => {}
      DefinitionContent::Lazy(_) => {}
    }
  }
}

#[derive(Default)]
struct FnState {
  mutable_this_established: bool,
  registers: HashMap<String, Value>,
}

impl FnState {
  fn clear(&mut self) {
    *self = Self::default();
  }

  fn simplify_line(&self, line: &mut FnLine) {
    match line {
      FnLine::Instruction(instr) => {
        if let Instruction::RequireMutableThis = instr {
          if self.mutable_this_established {
            *line = FnLine::Comment(line.to_string());
          }
        } else {
          instr.visit_fields_mut(&mut |field| match field {
            InstructionFieldMut::Value(arg) => {
              self.simplify_arg(arg);
            }
            _ => {}
          });
        }
      }
      FnLine::Label(..) | FnLine::Empty | FnLine::Comment(..) | FnLine::Release(..) => {}
    }
  }

  fn simplify_arg(&self, arg: &mut Value) {
    arg.visit_values_mut(&mut |value| {
      if let Value::Register(reg) = value {
        if let Some(new_value) = self.registers.get(&reg.name) {
          *value = new_value.clone();
        }
      }
    });
  }

  fn apply_line(&mut self, line: &FnLine) {
    match line {
      FnLine::Instruction(instr) => match instr {
        Instruction::End => {}
        Instruction::Mov(a1, dst) => {
          self.set_register(dst, Some(a1.clone()));
        }

        Instruction::OpInc(reg) => {
          // TODO: Use apply_binary_op?

          let new_value = match self.registers.get(&reg.name) {
            Some(Value::Number(Number(x))) => Some(Value::Number(Number(x + 1.0))),
            Some(Value::BigInt(x)) => Some(Value::BigInt(x + BigInt::from(1))),
            Some(_) | None => None,
          };

          self.set_register(reg, new_value);
        }
        Instruction::OpDec(reg) => {
          let new_value = match self.registers.get(&reg.name) {
            Some(Value::Number(Number(x))) => Some(Value::Number(Number(x - 1.0))),
            Some(Value::BigInt(x)) => Some(Value::BigInt(x - BigInt::from(1))),
            Some(_) | None => None,
          };

          self.set_register(reg, new_value);
        }

        Instruction::OpNot(a1, dst) => self.apply_unary_op(a1, dst, operations::op_not),
        Instruction::OpBitNot(a1, dst) => self.apply_unary_op(a1, dst, operations::op_bit_not),
        Instruction::TypeOf(a1, dst) => self.apply_unary_op(a1, dst, operations::op_typeof),
        Instruction::UnaryPlus(a1, dst) => self.apply_unary_op(a1, dst, operations::op_unary_plus),
        Instruction::UnaryMinus(a1, dst) => {
          self.apply_unary_op(a1, dst, operations::op_unary_minus)
        }
        Instruction::Import(_a1, dst)
        | Instruction::ImportStar(_a1, dst)
        | Instruction::Cat(_a1, dst) => {
          // TODO: cat
          self.set_register(dst, None);
        }

        Instruction::Yield(_a1, dst) | Instruction::YieldStar(_a1, dst) => {
          self.set_register(dst, None)
        }

        Instruction::Throw(_a1) => {}

        Instruction::OpPlus(a1, a2, dst) => self.apply_binary_op(a1, a2, dst, operations::op_plus),
        Instruction::OpMinus(a1, a2, dst) => {
          self.apply_binary_op(a1, a2, dst, operations::op_minus)
        }
        Instruction::OpMul(a1, a2, dst) => self.apply_binary_op(a1, a2, dst, operations::op_mul),
        Instruction::OpDiv(a1, a2, dst) => self.apply_binary_op(a1, a2, dst, operations::op_div),
        Instruction::OpMod(a1, a2, dst) => self.apply_binary_op(a1, a2, dst, operations::op_mod),
        Instruction::OpExp(a1, a2, dst) => self.apply_binary_op(a1, a2, dst, operations::op_exp),
        Instruction::OpEq(a1, a2, dst) => self.apply_binary_op(a1, a2, dst, operations::op_eq),
        Instruction::OpNe(a1, a2, dst) => self.apply_binary_op(a1, a2, dst, operations::op_ne),
        Instruction::OpTripleEq(a1, a2, dst) => {
          self.apply_binary_op(a1, a2, dst, operations::op_triple_eq)
        }
        Instruction::OpTripleNe(a1, a2, dst) => {
          self.apply_binary_op(a1, a2, dst, operations::op_triple_ne)
        }
        Instruction::OpAnd(a1, a2, dst) => self.apply_binary_op(a1, a2, dst, operations::op_and),
        Instruction::OpOr(a1, a2, dst) => self.apply_binary_op(a1, a2, dst, operations::op_or),
        Instruction::OpLess(a1, a2, dst) => self.apply_binary_op(a1, a2, dst, operations::op_less),
        Instruction::OpLessEq(a1, a2, dst) => {
          self.apply_binary_op(a1, a2, dst, operations::op_less_eq)
        }
        Instruction::OpGreater(a1, a2, dst) => {
          self.apply_binary_op(a1, a2, dst, operations::op_greater)
        }
        Instruction::OpGreaterEq(a1, a2, dst) => {
          self.apply_binary_op(a1, a2, dst, operations::op_greater_eq)
        }
        Instruction::OpNullishCoalesce(a1, a2, dst) => {
          self.apply_binary_op(a1, a2, dst, operations::op_nullish_coalesce)
        }
        Instruction::OpOptionalChain(_a1, _a2, dst) => {
          // self.apply_binary_op(a1, a2, dst, operations::op_optional_chain)
          // TODO: op_optional_chain takes mut lhs to optimize, but breaks this pattern
          self.set_register(dst, None);
        }
        Instruction::OpBitAnd(a1, a2, dst) => {
          self.apply_binary_op(a1, a2, dst, operations::op_bit_and)
        }
        Instruction::OpBitOr(a1, a2, dst) => {
          self.apply_binary_op(a1, a2, dst, operations::op_bit_or)
        }
        Instruction::OpBitXor(a1, a2, dst) => {
          self.apply_binary_op(a1, a2, dst, operations::op_bit_xor)
        }
        Instruction::OpLeftShift(a1, a2, dst) => {
          self.apply_binary_op(a1, a2, dst, operations::op_left_shift)
        }
        Instruction::OpRightShift(a1, a2, dst) => {
          self.apply_binary_op(a1, a2, dst, operations::op_right_shift)
        }
        Instruction::OpRightShiftUnsigned(a1, a2, dst) => {
          self.apply_binary_op(a1, a2, dst, operations::op_right_shift_unsigned)
        }
        Instruction::InstanceOf(a1, a2, dst) => {
          self.apply_binary_op(a1, a2, dst, operations::op_instance_of)
        }
        Instruction::In(a1, a2, dst) => self.apply_binary_op(a1, a2, dst, operations::op_in),

        Instruction::Call(_a1, _a2, dst)
        | Instruction::Bind(_a1, _a2, dst)
        | Instruction::Sub(_a1, _a2, dst)
        | Instruction::SubMov(_a1, _a2, dst)
        | Instruction::New(_a1, _a2, dst) => {
          self.set_register(dst, None);
        }

        Instruction::Apply(_a, this, _a3, dst)
        | Instruction::SubCall(this, _a, _a3, dst)
        | Instruction::ThisSubCall(this, _a, _a3, dst) => {
          self.set_register(this, None);
          self.set_register(dst, None);
        }

        Instruction::ConstSubCall(_a1, _a2, _a3, dst) => self.set_register(dst, None),

        Instruction::JmpIf(_a1, _) => {}

        Instruction::Jmp(_) => {}
        Instruction::SetCatch(_, _) => {}
        Instruction::UnsetCatch => {}
        Instruction::RequireMutableThis => {
          self.mutable_this_established = true;
        }
        Instruction::Next(iter, dst) => {
          self.set_register(iter, None);
          self.set_register(dst, None);
        }
        Instruction::UnpackIterRes(iter_res, value_reg, done) => {
          self.set_register(iter_res, None);
          self.set_register(value_reg, None);
          self.set_register(done, None);
        }
      },
      FnLine::Label(..) => self.clear(),
      FnLine::Empty | FnLine::Comment(..) => {}
      FnLine::Release(reg) => {
        self.set_register(reg, None);
      }
    }
  }

  fn set_register(&mut self, reg: &Register, value: Option<Value>) {
    let mut registers_to_clear = HashSet::<String>::new();

    for (k, v) in &mut self.registers {
      v.visit_values_mut(&mut |value| {
        if let Value::Register(reg_value) = value {
          if reg_value.name == reg.name {
            registers_to_clear.insert(k.clone());
          }
        }
      });
    }

    for reg_to_clear in registers_to_clear {
      self.registers.remove(&reg_to_clear);
    }

    match value {
      Some(value) => self.registers.insert(reg.name.clone(), value),
      None => self.registers.remove(&reg.name),
    };
  }

  fn apply_unary_op(&mut self, arg: &Value, dst: &Register, op: fn(input: &Val) -> Val) {
    match self.apply_unary_op_impl(arg, dst, op) {
      Ok(_) => {}
      Err(_) => {
        self.set_register(dst, None);
      }
    }
  }

  fn apply_unary_op_impl(
    &mut self,
    arg: &Value,
    dst: &Register,
    op: fn(input: &Val) -> Val,
  ) -> Result<(), Val> {
    let arg = arg.clone().try_to_val()?;
    let value = op(&arg).try_to_value()?;

    self.set_register(dst, Some(value));

    Ok(())
  }

  fn apply_binary_op(
    &mut self,
    left: &Value,
    right: &Value,
    dst: &Register,
    op: fn(left: &Val, right: &Val) -> Result<Val, Val>,
  ) {
    match self.apply_binary_op_impl(left, right, dst, op) {
      Ok(_) => {}
      Err(_) => {
        self.set_register(dst, None);
      }
    }
  }

  fn apply_binary_op_impl(
    &mut self,
    left: &Value,
    right: &Value,
    dst: &Register,
    op: fn(left: &Val, right: &Val) -> Result<Val, Val>,
  ) -> Result<(), Val> {
    let left = left.clone().try_to_val()?;
    let right = right.clone().try_to_val()?;
    let value = op(&left, &right)?.try_to_value()?;

    self.set_register(dst, Some(value));

    Ok(())
  }

  fn handle_releases(&self, body: &mut Vec<FnLine>, i: usize) {
    let mut calls = Vec::<(Register, usize)>::new();

    match &mut body[i] {
      FnLine::Instruction(instr) => {
        let mut skips_needed = 0;

        instr.visit_registers_mut_rev(&mut |rvm| {
          skips_needed += 1;

          if rvm.write && !rvm.read {
            calls.push((rvm.register.clone(), skips_needed));
          }
        });
      }
      FnLine::Release(released_reg) => {
        calls.push((released_reg.clone(), 0));
      }
      FnLine::Label(_) | FnLine::Empty | FnLine::Comment(_) => {}
    };

    for (released_reg, skips) in calls {
      self.handle_releases_impl(body, i, released_reg, skips);
    }
  }

  fn handle_releases_impl(
    &self,
    body: &mut Vec<FnLine>,
    i: usize,
    released_reg: Register,
    skips_needed: usize,
  ) {
    // Search backwards to find where this register was last written. If a jump instruction occurs,
    // then we don't know for sure whether the release point will be hit, and we can't apply our
    // analysis.
    let mut j = i + 1;
    let mut skips = 0;
    while j > 0 {
      j -= 1;

      let instr = match &mut body[j] {
        FnLine::Instruction(instr) => instr,
        _ => continue,
      };

      if is_jmp_instr(instr) {
        return;
      }

      let mut write_found = false;

      instr.visit_registers_mut_rev(&mut |rvm| {
        if skips < skips_needed {
          skips += 1;
          return;
        }

        if rvm.write && rvm.register.name == released_reg.name {
          write_found = true;
        }
      });

      if write_found {
        break;
      }
    }

    // Now that we've established that the last write always hits the release point, find the last
    // read and use .take() instead of copying. Also, if this .take() never occurs, it means the
    // value was never used, and comment out the instruction that writes the value, if possible.
    let mut j = i + 1;
    let mut skip_i = 0;
    let mut taken = false;
    while j > 0 {
      j -= 1;

      let instr = match &mut body[j] {
        FnLine::Instruction(instr) => instr,
        _ => continue,
      };

      let mut write_found = false;

      if !taken {
        instr.visit_registers_mut_rev(&mut |rvm| {
          if skip_i < skips_needed {
            skip_i += 1;
            return;
          }

          if rvm.register.name != released_reg.name {
            return;
          }

          if !taken && !rvm.write {
            *rvm.register = rvm.register.take();
            taken = true;
          }

          if !write_found && rvm.write {
            write_found = true;

            if !rvm.read && !taken {
              *rvm.register = Register::ignore();
            }
          }
        });
      }

      if write_found {
        break;
      }
    }
  }
}

fn simplify_fn(mut state: FnState, fn_: &mut Function) {
  for i in 0..fn_.body.len() {
    let line = &mut fn_.body[i];

    state.simplify_line(line);
    state.apply_line(line);

    state.handle_releases(&mut fn_.body, i);
  }
}

fn is_jmp_instr(instr: &Instruction) -> bool {
  match instr {
    Instruction::End | Instruction::Jmp(..) | Instruction::JmpIf(..) => true,
    Instruction::Mov(..)
    | Instruction::OpInc(..)
    | Instruction::OpDec(..)
    | Instruction::OpPlus(..)
    | Instruction::OpMinus(..)
    | Instruction::OpMul(..)
    | Instruction::OpDiv(..)
    | Instruction::OpMod(..)
    | Instruction::OpExp(..)
    | Instruction::OpEq(..)
    | Instruction::OpNe(..)
    | Instruction::OpTripleEq(..)
    | Instruction::OpTripleNe(..)
    | Instruction::OpAnd(..)
    | Instruction::OpOr(..)
    | Instruction::OpNot(..)
    | Instruction::OpLess(..)
    | Instruction::OpLessEq(..)
    | Instruction::OpGreater(..)
    | Instruction::OpGreaterEq(..)
    | Instruction::OpNullishCoalesce(..)
    | Instruction::OpOptionalChain(..)
    | Instruction::OpBitAnd(..)
    | Instruction::OpBitOr(..)
    | Instruction::OpBitNot(..)
    | Instruction::OpBitXor(..)
    | Instruction::OpLeftShift(..)
    | Instruction::OpRightShift(..)
    | Instruction::OpRightShiftUnsigned(..)
    | Instruction::TypeOf(..)
    | Instruction::InstanceOf(..)
    | Instruction::In(..)
    | Instruction::Call(..)
    | Instruction::Apply(..)
    | Instruction::Bind(..)
    | Instruction::Sub(..)
    | Instruction::SubMov(..)
    | Instruction::SubCall(..)
    | Instruction::UnaryPlus(..)
    | Instruction::UnaryMinus(..)
    | Instruction::New(..)
    | Instruction::Throw(..)
    | Instruction::Import(..)
    | Instruction::ImportStar(..)
    | Instruction::SetCatch(..)
    | Instruction::UnsetCatch
    | Instruction::ConstSubCall(..)
    | Instruction::RequireMutableThis
    | Instruction::ThisSubCall(..)
    | Instruction::Next(..)
    | Instruction::UnpackIterRes(..)
    | Instruction::Cat(..)
    | Instruction::Yield(..)
    | Instruction::YieldStar(..) => false,
  }
}
