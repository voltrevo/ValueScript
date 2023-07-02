use std::mem::take;

use crate::{
  asm::{DefinitionContent, FnLine, Module, Value},
  instruction::Instruction,
};

pub fn reduce_instructions(module: &mut Module) {
  for defn in &mut module.definitions {
    if let DefinitionContent::Function(fn_) = &mut defn.content {
      for line in take(&mut fn_.body) {
        if let FnLine::Instruction(instr) = line {
          if let Some(instr) = reduce(instr) {
            fn_.body.push(FnLine::Instruction(instr));
          }
        } else {
          fn_.body.push(line);
        }
      }
    }
  }
}

fn reduce(instr: Instruction) -> Option<Instruction> {
  use Instruction::*;

  match &instr {
    End | OpInc(..) | OpDec(..) | Call(..) | Apply(..) | SubCall(..) | Jmp(..) | New(..)
    | Throw(..) | SetCatch(..) | UnsetCatch | ConstSubCall(..) | RequireMutableThis
    | ThisSubCall(..) | Next(..) | Yield(..) | YieldStar(..) => Some(instr),

    Mov(_, dst)
    | OpPlus(_, _, dst)
    | OpMinus(_, _, dst)
    | OpMul(_, _, dst)
    | OpDiv(_, _, dst)
    | OpMod(_, _, dst)
    | OpExp(_, _, dst)
    | OpEq(_, _, dst)
    | OpNe(_, _, dst)
    | OpTripleEq(_, _, dst)
    | OpTripleNe(_, _, dst)
    | OpAnd(_, _, dst)
    | OpOr(_, _, dst)
    | OpNot(_, dst)
    | OpLess(_, _, dst)
    | OpLessEq(_, _, dst)
    | OpGreater(_, _, dst)
    | OpGreaterEq(_, _, dst)
    | OpNullishCoalesce(_, _, dst)
    | OpOptionalChain(_, _, dst)
    | OpBitAnd(_, _, dst)
    | OpBitOr(_, _, dst)
    | OpBitNot(_, dst)
    | OpBitXor(_, _, dst)
    | OpLeftShift(_, _, dst)
    | OpRightShift(_, _, dst)
    | OpRightShiftUnsigned(_, _, dst)
    | TypeOf(_, dst)
    | InstanceOf(_, _, dst)
    | In(_, _, dst)
    | Bind(_, _, dst)
    | Sub(_, _, dst)
    | SubMov(_, _, dst)
    | UnaryPlus(_, dst)
    | UnaryMinus(_, dst)
    | Import(_, dst)
    | ImportStar(_, dst)
    | Cat(_, dst) => {
      if dst.is_ignore() {
        None
      } else {
        Some(instr)
      }
    }

    JmpIf(cond, label_ref) => match cond {
      // TODO: Kal::from_value(cond).is_truthy()
      Value::Bool(false) => None,
      Value::Bool(true) => Some(Instruction::Jmp(label_ref.clone())),
      _ => Some(instr),
    },
    JmpIfNot(cond, label_ref) => match cond {
      // TODO: Kal::from_value(cond).is_truthy()
      Value::Bool(false) => Some(Instruction::Jmp(label_ref.clone())),
      Value::Bool(true) => None,
      _ => Some(instr),
    },
    UnpackIterRes(_, value_dst, done_dst) => {
      if value_dst.is_ignore() && done_dst.is_ignore() {
        None
      } else {
        Some(instr)
      }
    }
  }
}
