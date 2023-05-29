use crate::asm::{Instruction, Register, Value};

pub fn instruction_mutates_this(instruction: &Instruction) -> bool {
  use Instruction::*;

  match instruction {
    End | Jmp(..) | JmpIf(..) | Throw(..) | UnsetCatch | RequireMutableThis => false,
    Mov(_, reg)
    | OpInc(reg)
    | OpDec(reg)
    | OpPlus(_, _, reg)
    | OpMinus(_, _, reg)
    | OpMul(_, _, reg)
    | OpDiv(_, _, reg)
    | OpMod(_, _, reg)
    | OpExp(_, _, reg)
    | OpEq(_, _, reg)
    | OpNe(_, _, reg)
    | OpTripleEq(_, _, reg)
    | OpTripleNe(_, _, reg)
    | OpAnd(_, _, reg)
    | OpOr(_, _, reg)
    | OpNot(_, reg)
    | OpLess(_, _, reg)
    | OpLessEq(_, _, reg)
    | OpGreater(_, _, reg)
    | OpGreaterEq(_, _, reg)
    | OpNullishCoalesce(_, _, reg)
    | OpOptionalChain(_, _, reg)
    | OpBitAnd(_, _, reg)
    | OpBitOr(_, _, reg)
    | OpBitNot(_, reg)
    | OpBitXor(_, _, reg)
    | OpLeftShift(_, _, reg)
    | OpRightShift(_, _, reg)
    | OpRightShiftUnsigned(_, _, reg)
    | TypeOf(_, reg)
    | InstanceOf(_, _, reg)
    | In(_, _, reg)
    | Call(_, _, reg)
    | Bind(_, _, reg)
    | Sub(_, _, reg)
    | SubMov(_, _, reg)
    | UnaryPlus(_, reg)
    | UnaryMinus(_, reg)
    | New(_, _, reg)
    | Import(_, reg)
    | ImportStar(_, reg)
    | SetCatch(_, reg)
    | ConstSubCall(_, _, _, reg)
    | ThisSubCall(_, _, _, reg) => reg == &Register::This,

    Next(iter, res) => iter == &Register::This || res == &Register::This,
    UnpackIterRes(_, value_reg, done_reg) => {
      value_reg == &Register::This || done_reg == &Register::This
    }

    Apply(_, ctx, _, reg) | SubCall(ctx, _, _, reg) => {
      reg == &Register::This
        || match ctx {
          Value::Register(reg) => reg == &Register::This,
          _ => false,
        }
    }
  }
}
