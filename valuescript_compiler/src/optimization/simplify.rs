use std::collections::{HashMap, HashSet};

use num_bigint::BigInt;
use valuescript_vm::{operations, vs_value::Val};

use crate::{
  asm::{DefinitionContent, FnLine, Function, Instruction, Module, Number, Register, Value},
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
      FnLine::Instruction(instr) => match instr {
        Instruction::End => {}
        Instruction::Mov(a1, _) => {
          self.simplify_arg(a1);
        }

        Instruction::OpInc(_) => {}
        Instruction::OpDec(_) => {}

        Instruction::OpNot(a1, _)
        | Instruction::OpBitNot(a1, _)
        | Instruction::TypeOf(a1, _)
        | Instruction::UnaryPlus(a1, _)
        | Instruction::UnaryMinus(a1, _)
        | Instruction::Throw(a1)
        | Instruction::Import(a1, _)
        | Instruction::ImportStar(a1, _)
        | Instruction::Cat(a1, _)
        | Instruction::Yield(a1, _)
        | Instruction::YieldStar(a1, _) => {
          self.simplify_arg(a1);
        }

        Instruction::OpPlus(a1, a2, _)
        | Instruction::OpMinus(a1, a2, _)
        | Instruction::OpMul(a1, a2, _)
        | Instruction::OpDiv(a1, a2, _)
        | Instruction::OpMod(a1, a2, _)
        | Instruction::OpExp(a1, a2, _)
        | Instruction::OpEq(a1, a2, _)
        | Instruction::OpNe(a1, a2, _)
        | Instruction::OpTripleEq(a1, a2, _)
        | Instruction::OpTripleNe(a1, a2, _)
        | Instruction::OpAnd(a1, a2, _)
        | Instruction::OpOr(a1, a2, _)
        | Instruction::OpLess(a1, a2, _)
        | Instruction::OpLessEq(a1, a2, _)
        | Instruction::OpGreater(a1, a2, _)
        | Instruction::OpGreaterEq(a1, a2, _)
        | Instruction::OpNullishCoalesce(a1, a2, _)
        | Instruction::OpOptionalChain(a1, a2, _)
        | Instruction::OpBitAnd(a1, a2, _)
        | Instruction::OpBitOr(a1, a2, _)
        | Instruction::OpBitXor(a1, a2, _)
        | Instruction::OpLeftShift(a1, a2, _)
        | Instruction::OpRightShift(a1, a2, _)
        | Instruction::OpRightShiftUnsigned(a1, a2, _)
        | Instruction::InstanceOf(a1, a2, _)
        | Instruction::In(a1, a2, _)
        | Instruction::Call(a1, a2, _)
        | Instruction::Bind(a1, a2, _)
        | Instruction::Sub(a1, a2, _)
        | Instruction::SubMov(a1, a2, _)
        | Instruction::New(a1, a2, _) => {
          self.simplify_arg(a1);
          self.simplify_arg(a2);
        }

        Instruction::Apply(a1, _this, a3, _) => {
          self.simplify_arg(a1);
          self.simplify_arg(a3);
        }

        Instruction::SubCall(_this, a2, a3, _) | Instruction::ThisSubCall(_this, a2, a3, _) => {
          self.simplify_arg(a2);
          self.simplify_arg(a3);
        }

        Instruction::ConstSubCall(a1, a2, a3, _) => {
          self.simplify_arg(a1);
          self.simplify_arg(a2);
          self.simplify_arg(a3);
        }

        Instruction::JmpIf(a1, _) => {
          self.simplify_arg(a1);
        }

        Instruction::Jmp(_) => {}
        Instruction::SetCatch(_, _) => {}
        Instruction::UnsetCatch => {}
        Instruction::RequireMutableThis => {
          if self.mutable_this_established {
            *line = FnLine::Comment(line.to_string());
          }
        }
        Instruction::Next(_, _) => {}
        Instruction::UnpackIterRes(_, _, _) => {}
      },
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
          if let Value::Register(this) = this {
            self.set_register(this, None);
          }

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
      FnLine::Release(_reg) => {
        // TODO
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
}

fn simplify_fn(mut state: FnState, fn_: &mut Function) {
  for line in &mut fn_.body {
    state.simplify_line(line);
    state.apply_line(line);
  }
}
