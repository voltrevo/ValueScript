use valuescript_common::InstructionByte;

use crate::asm::{LabelRef, Register, StructuredFormattable, StructuredFormatter, Value};

#[derive(Debug, Clone)]
pub enum Instruction {
  End,
  Mov(Value, Register),
  OpInc(Register),
  OpDec(Register),
  OpPlus(Value, Value, Register),
  OpMinus(Value, Value, Register),
  OpMul(Value, Value, Register),
  OpDiv(Value, Value, Register),
  OpMod(Value, Value, Register),
  OpExp(Value, Value, Register),
  OpEq(Value, Value, Register),
  OpNe(Value, Value, Register),
  OpTripleEq(Value, Value, Register),
  OpTripleNe(Value, Value, Register),
  OpAnd(Value, Value, Register),
  OpOr(Value, Value, Register),
  OpNot(Value, Register),
  OpLess(Value, Value, Register),
  OpLessEq(Value, Value, Register),
  OpGreater(Value, Value, Register),
  OpGreaterEq(Value, Value, Register),
  OpNullishCoalesce(Value, Value, Register),
  OpOptionalChain(Value, Value, Register),
  OpBitAnd(Value, Value, Register),
  OpBitOr(Value, Value, Register),
  OpBitNot(Value, Register),
  OpBitXor(Value, Value, Register),
  OpLeftShift(Value, Value, Register),
  OpRightShift(Value, Value, Register),
  OpRightShiftUnsigned(Value, Value, Register),
  TypeOf(Value, Register),
  InstanceOf(Value, Value, Register),
  In(Value, Value, Register),
  Call(Value, Value, Register),
  Apply(Value, Register, Value, Register),
  ConstApply(Value, Value, Value, Register),
  Bind(Value, Value, Register),
  Sub(Value, Value, Register),
  SubMov(Value, Value, Register),
  SubCall(Register, Value, Value, Register),
  Jmp(LabelRef),
  JmpIf(Value, LabelRef),
  JmpIfNot(Value, LabelRef),
  UnaryPlus(Value, Register),
  UnaryMinus(Value, Register),
  New(Value, Value, Register),
  Throw(Value),
  Import(Value, Register),
  ImportStar(Value, Register),
  SetCatch(LabelRef, Register),
  UnsetCatch,
  ConstSubCall(Value, Value, Value, Register),
  RequireMutableThis,
  ThisSubCall(Register, Value, Value, Register),
  Next(Register, Register),
  UnpackIterRes(Register, Register, Register),
  Cat(Value, Register),
  Yield(Value, Register),
  YieldStar(Value, Register),
  Delete(Register, Value, Register),
}

pub enum InstructionFieldMut<'a> {
  Value(&'a mut Value),
  Register(&'a mut Register),
  LabelRef(&'a mut LabelRef),
}

pub struct RegisterVisitMut<'a> {
  pub register: &'a mut Register,
  pub read: bool,
  pub write: bool,
}

impl<'a> RegisterVisitMut<'a> {
  pub fn read(register: &'a mut Register) -> Self {
    RegisterVisitMut {
      register,
      read: true,
      write: false,
    }
  }

  pub fn write(register: &'a mut Register) -> Self {
    RegisterVisitMut {
      register,
      read: false,
      write: true,
    }
  }

  pub fn read_and_write(register: &'a mut Register) -> Self {
    RegisterVisitMut {
      register,
      read: true,
      write: true,
    }
  }
}

impl Instruction {
  pub fn visit_fields_mut<F>(&mut self, visit: &mut F)
  where
    F: FnMut(InstructionFieldMut),
  {
    use Instruction::*;

    match self {
      End => {}
      Mov(arg, dst)
      | OpNot(arg, dst)
      | OpBitNot(arg, dst)
      | TypeOf(arg, dst)
      | UnaryPlus(arg, dst)
      | UnaryMinus(arg, dst)
      | Import(arg, dst)
      | ImportStar(arg, dst)
      | Cat(arg, dst)
      | Yield(arg, dst)
      | YieldStar(arg, dst) => {
        visit(InstructionFieldMut::Value(arg));
        visit(InstructionFieldMut::Register(dst));
      }

      OpInc(arg) | OpDec(arg) => {
        visit(InstructionFieldMut::Register(arg));
      }

      OpPlus(left, right, dst)
      | OpMinus(left, right, dst)
      | OpMul(left, right, dst)
      | OpDiv(left, right, dst)
      | OpMod(left, right, dst)
      | OpExp(left, right, dst)
      | OpEq(left, right, dst)
      | OpNe(left, right, dst)
      | OpTripleEq(left, right, dst)
      | OpTripleNe(left, right, dst)
      | OpAnd(left, right, dst)
      | OpOr(left, right, dst)
      | OpLess(left, right, dst)
      | OpLessEq(left, right, dst)
      | OpGreater(left, right, dst)
      | OpGreaterEq(left, right, dst)
      | OpNullishCoalesce(left, right, dst)
      | OpOptionalChain(left, right, dst)
      | OpBitAnd(left, right, dst)
      | OpBitOr(left, right, dst)
      | OpBitXor(left, right, dst)
      | OpLeftShift(left, right, dst)
      | OpRightShift(left, right, dst)
      | OpRightShiftUnsigned(left, right, dst)
      | InstanceOf(left, right, dst)
      | In(left, right, dst)
      | Call(left, right, dst)
      | Bind(left, right, dst)
      | Sub(left, right, dst)
      | SubMov(left, right, dst)
      | New(left, right, dst) => {
        visit(InstructionFieldMut::Value(left));
        visit(InstructionFieldMut::Value(right));
        visit(InstructionFieldMut::Register(dst));
      }

      Apply(fn_, this, args, dst) => {
        visit(InstructionFieldMut::Value(fn_));
        visit(InstructionFieldMut::Register(this));
        visit(InstructionFieldMut::Value(args));
        visit(InstructionFieldMut::Register(dst));
      }

      ConstSubCall(a1, a2, a3, dst) | ConstApply(a1, a2, a3, dst) => {
        visit(InstructionFieldMut::Value(a1));
        visit(InstructionFieldMut::Value(a2));
        visit(InstructionFieldMut::Value(a3));
        visit(InstructionFieldMut::Register(dst));
      }

      SubCall(this, key, args, dst) | ThisSubCall(this, key, args, dst) => {
        visit(InstructionFieldMut::Register(this));
        visit(InstructionFieldMut::Value(key));
        visit(InstructionFieldMut::Value(args));
        visit(InstructionFieldMut::Register(dst));
      }

      Jmp(label_ref) => {
        visit(InstructionFieldMut::LabelRef(label_ref));
      }

      JmpIf(cond, label_ref) | JmpIfNot(cond, label_ref) => {
        visit(InstructionFieldMut::Value(cond));
        visit(InstructionFieldMut::LabelRef(label_ref));
      }

      Throw(ex) => {
        visit(InstructionFieldMut::Value(ex));
      }

      SetCatch(label_ref, dst) => {
        visit(InstructionFieldMut::LabelRef(label_ref));
        visit(InstructionFieldMut::Register(dst));
      }

      Next(iterable, dst) => {
        visit(InstructionFieldMut::Register(iterable));
        visit(InstructionFieldMut::Register(dst));
      }

      UnpackIterRes(iter_res, value_dst, done_dst) => {
        visit(InstructionFieldMut::Register(iter_res));
        visit(InstructionFieldMut::Register(value_dst));
        visit(InstructionFieldMut::Register(done_dst));
      }

      UnsetCatch | RequireMutableThis => {}

      Delete(obj, sub, dst) => {
        visit(InstructionFieldMut::Register(obj));
        visit(InstructionFieldMut::Value(sub));
        visit(InstructionFieldMut::Register(dst));
      }
    }
  }

  pub fn visit_registers_mut_rev<F>(&mut self, visit: &mut F)
  where
    F: FnMut(RegisterVisitMut),
  {
    use Instruction::*;

    match self {
      End => {}
      Mov(arg, dst)
      | OpNot(arg, dst)
      | OpBitNot(arg, dst)
      | TypeOf(arg, dst)
      | UnaryPlus(arg, dst)
      | UnaryMinus(arg, dst)
      | Import(arg, dst)
      | ImportStar(arg, dst)
      | Cat(arg, dst)
      | Yield(arg, dst)
      | YieldStar(arg, dst) => {
        visit(RegisterVisitMut::write(dst));
        arg.visit_registers_mut_rev(visit);
      }

      OpInc(arg) | OpDec(arg) => {
        visit(RegisterVisitMut::read_and_write(arg));
      }

      OpPlus(left, right, dst)
      | OpMinus(left, right, dst)
      | OpMul(left, right, dst)
      | OpDiv(left, right, dst)
      | OpMod(left, right, dst)
      | OpExp(left, right, dst)
      | OpEq(left, right, dst)
      | OpNe(left, right, dst)
      | OpTripleEq(left, right, dst)
      | OpTripleNe(left, right, dst)
      | OpAnd(left, right, dst)
      | OpOr(left, right, dst)
      | OpLess(left, right, dst)
      | OpLessEq(left, right, dst)
      | OpGreater(left, right, dst)
      | OpGreaterEq(left, right, dst)
      | OpNullishCoalesce(left, right, dst)
      | OpOptionalChain(left, right, dst)
      | OpBitAnd(left, right, dst)
      | OpBitOr(left, right, dst)
      | OpBitXor(left, right, dst)
      | OpLeftShift(left, right, dst)
      | OpRightShift(left, right, dst)
      | OpRightShiftUnsigned(left, right, dst)
      | InstanceOf(left, right, dst)
      | In(left, right, dst)
      | Call(left, right, dst)
      | Bind(left, right, dst)
      | Sub(left, right, dst)
      | New(left, right, dst) => {
        visit(RegisterVisitMut::write(dst));
        right.visit_registers_mut_rev(visit);
        left.visit_registers_mut_rev(visit);
      }

      SubMov(key, value, obj) => {
        // `obj` is 'read' here in the sense that it needs to be intact for this to work, it's not
        // just newly constructed from the inputs.
        visit(RegisterVisitMut::read_and_write(obj));

        value.visit_registers_mut_rev(visit);
        key.visit_registers_mut_rev(visit);
      }

      Apply(fn_, this, args, dst) => {
        visit(RegisterVisitMut::write(dst));
        args.visit_registers_mut_rev(visit);
        visit(RegisterVisitMut::read_and_write(this));
        fn_.visit_registers_mut_rev(visit);
      }

      ConstSubCall(a1, a2, a3, dst) | ConstApply(a1, a2, a3, dst) => {
        visit(RegisterVisitMut::write(dst));
        a3.visit_registers_mut_rev(visit);
        a2.visit_registers_mut_rev(visit);
        a1.visit_registers_mut_rev(visit);
      }

      SubCall(this, key, args, dst) | ThisSubCall(this, key, args, dst) => {
        visit(RegisterVisitMut::write(dst));
        args.visit_registers_mut_rev(visit);
        key.visit_registers_mut_rev(visit);
        visit(RegisterVisitMut::read_and_write(this));
      }

      Jmp(_label_ref) => {}

      JmpIf(cond, _label_ref) | JmpIfNot(cond, _label_ref) => {
        cond.visit_registers_mut_rev(visit);
      }

      Throw(ex) => {
        ex.visit_registers_mut_rev(visit);
      }

      SetCatch(_label_ref, dst) => {
        // TODO: There is a write but it doesn't occur here. There should really be a write at the
        // catch label. This is only ok right now because Kals currently get cleared on labels.
        visit(RegisterVisitMut {
          register: dst,
          read: false,
          write: false,
        });
      }

      Next(iterable, dst) => {
        visit(RegisterVisitMut::write(dst));
        visit(RegisterVisitMut::read_and_write(iterable));
      }

      UnpackIterRes(iter_res, value_dst, done_dst) => {
        visit(RegisterVisitMut::write(done_dst));
        visit(RegisterVisitMut::write(value_dst));

        // This does a write in the sense that the register is consumed. However, the new value
        // being written is not supposed to matter, `unpack_iter_res` should only be used once and
        // the emptiness of the register isn't supposed to be then consumed the way a regular write
        // would. In other words, well-constructed programs should be equivalent whether the VM
        // decides to consume this memory or not.
        visit(RegisterVisitMut::read(iter_res));
      }

      UnsetCatch | RequireMutableThis => {}

      Delete(obj, sub, dst) => {
        visit(RegisterVisitMut::read_and_write(obj));
        sub.visit_registers_mut_rev(visit);
        visit(RegisterVisitMut::write(dst));
      }
    }
  }

  pub fn byte(&self) -> InstructionByte {
    use Instruction::*;

    // TODO: Define this in one place only
    match self {
      End => InstructionByte::End,
      Mov(..) => InstructionByte::Mov,
      OpInc(..) => InstructionByte::OpInc,
      OpDec(..) => InstructionByte::OpDec,
      OpPlus(..) => InstructionByte::OpPlus,
      OpMinus(..) => InstructionByte::OpMinus,
      OpMul(..) => InstructionByte::OpMul,
      OpDiv(..) => InstructionByte::OpDiv,
      OpMod(..) => InstructionByte::OpMod,
      OpExp(..) => InstructionByte::OpExp,
      OpEq(..) => InstructionByte::OpEq,
      OpNe(..) => InstructionByte::OpNe,
      OpTripleEq(..) => InstructionByte::OpTripleEq,
      OpTripleNe(..) => InstructionByte::OpTripleNe,
      OpAnd(..) => InstructionByte::OpAnd,
      OpOr(..) => InstructionByte::OpOr,
      OpNot(..) => InstructionByte::OpNot,
      OpLess(..) => InstructionByte::OpLess,
      OpLessEq(..) => InstructionByte::OpLessEq,
      OpGreater(..) => InstructionByte::OpGreater,
      OpGreaterEq(..) => InstructionByte::OpGreaterEq,
      OpNullishCoalesce(..) => InstructionByte::OpNullishCoalesce,
      OpOptionalChain(..) => InstructionByte::OpOptionalChain,
      OpBitAnd(..) => InstructionByte::OpBitAnd,
      OpBitOr(..) => InstructionByte::OpBitOr,
      OpBitNot(..) => InstructionByte::OpBitNot,
      OpBitXor(..) => InstructionByte::OpBitXor,
      OpLeftShift(..) => InstructionByte::OpLeftShift,
      OpRightShift(..) => InstructionByte::OpRightShift,
      OpRightShiftUnsigned(..) => InstructionByte::OpRightShiftUnsigned,
      TypeOf(..) => InstructionByte::TypeOf,
      InstanceOf(..) => InstructionByte::InstanceOf,
      In(..) => InstructionByte::In,
      Call(..) => InstructionByte::Call,
      Apply(..) => InstructionByte::Apply,
      ConstApply(..) => InstructionByte::ConstApply,
      Bind(..) => InstructionByte::Bind,
      Sub(..) => InstructionByte::Sub,
      SubMov(..) => InstructionByte::SubMov,
      SubCall(..) => InstructionByte::SubCall,
      Jmp(..) => InstructionByte::Jmp,
      JmpIf(..) => InstructionByte::JmpIf,
      JmpIfNot(..) => InstructionByte::JmpIfNot,
      UnaryPlus(..) => InstructionByte::UnaryPlus,
      UnaryMinus(..) => InstructionByte::UnaryMinus,
      New(..) => InstructionByte::New,
      Throw(..) => InstructionByte::Throw,
      Import(..) => InstructionByte::Import,
      ImportStar(..) => InstructionByte::ImportStar,
      SetCatch(..) => InstructionByte::SetCatch,
      UnsetCatch => InstructionByte::UnsetCatch,
      ConstSubCall(..) => InstructionByte::ConstSubCall,
      RequireMutableThis => InstructionByte::RequireMutableThis,
      ThisSubCall(..) => InstructionByte::ThisSubCall,
      Next(..) => InstructionByte::Next,
      UnpackIterRes(..) => InstructionByte::UnpackIterRes,
      Cat(..) => InstructionByte::Cat,
      Yield(..) => InstructionByte::Yield,
      YieldStar(..) => InstructionByte::YieldStar,
      Delete(..) => InstructionByte::Delete,
    }
  }
}

impl StructuredFormattable for Instruction {
  fn structured_fmt(&self, sf: &mut StructuredFormatter<'_, '_>) -> std::fmt::Result {
    match self {
      Instruction::End => sf.write("end"),
      Instruction::Mov(value, register) => sf.write_slice_joined(" ", &[&"mov", value, register]),
      Instruction::OpInc(register) => sf.write_slice_joined(" ", &[&"op++", register]),
      Instruction::OpDec(register) => sf.write_slice_joined(" ", &[&"op--", register]),
      Instruction::OpPlus(lhs, rhs, register) => {
        sf.write_slice_joined(" ", &[&"op+", lhs, rhs, register])
      }
      Instruction::OpMinus(lhs, rhs, register) => {
        sf.write_slice_joined(" ", &[&"op-", lhs, rhs, register])
      }
      Instruction::OpMul(lhs, rhs, register) => {
        sf.write_slice_joined(" ", &[&"op*", lhs, rhs, register])
      }
      Instruction::OpDiv(lhs, rhs, register) => {
        sf.write_slice_joined(" ", &[&"op/", lhs, rhs, register])
      }
      Instruction::OpMod(lhs, rhs, register) => {
        sf.write_slice_joined(" ", &[&"op%", lhs, rhs, register])
      }
      Instruction::OpExp(lhs, rhs, register) => {
        sf.write_slice_joined(" ", &[&"op**", lhs, rhs, register])
      }
      Instruction::OpEq(lhs, rhs, register) => {
        sf.write_slice_joined(" ", &[&"op==", lhs, rhs, register])
      }
      Instruction::OpNe(lhs, rhs, register) => {
        sf.write_slice_joined(" ", &[&"op!=", lhs, rhs, register])
      }
      Instruction::OpTripleEq(lhs, rhs, register) => {
        sf.write_slice_joined(" ", &[&"op===", lhs, rhs, register])
      }
      Instruction::OpTripleNe(lhs, rhs, register) => {
        sf.write_slice_joined(" ", &[&"op!==", lhs, rhs, register])
      }
      Instruction::OpAnd(lhs, rhs, register) => {
        sf.write_slice_joined(" ", &[&"op&&", lhs, rhs, register])
      }
      Instruction::OpOr(lhs, rhs, register) => {
        sf.write_slice_joined(" ", &[&"op||", lhs, rhs, register])
      }
      Instruction::OpNot(value, register) => sf.write_slice_joined(" ", &[&"op!", value, register]),
      Instruction::OpLess(lhs, rhs, register) => {
        sf.write_slice_joined(" ", &[&"op<", lhs, rhs, register])
      }
      Instruction::OpLessEq(lhs, rhs, register) => {
        sf.write_slice_joined(" ", &[&"op<=", lhs, rhs, register])
      }
      Instruction::OpGreater(lhs, rhs, register) => {
        sf.write_slice_joined(" ", &[&"op>", lhs, rhs, register])
      }
      Instruction::OpGreaterEq(lhs, rhs, register) => {
        sf.write_slice_joined(" ", &[&"op>=", lhs, rhs, register])
      }
      Instruction::OpNullishCoalesce(lhs, rhs, register) => {
        sf.write_slice_joined(" ", &[&"op??", lhs, rhs, register])
      }
      Instruction::OpOptionalChain(lhs, rhs, register) => {
        sf.write_slice_joined(" ", &[&"op?.", lhs, rhs, register])
      }
      Instruction::OpBitAnd(lhs, rhs, register) => {
        sf.write_slice_joined(" ", &[&"op&", lhs, rhs, register])
      }
      Instruction::OpBitOr(lhs, rhs, register) => {
        sf.write_slice_joined(" ", &[&"op|", lhs, rhs, register])
      }
      Instruction::OpBitNot(value, register) => {
        sf.write_slice_joined(" ", &[&"op~", value, register])
      }
      Instruction::OpBitXor(lhs, rhs, register) => {
        sf.write_slice_joined(" ", &[&"op^", lhs, rhs, register])
      }
      Instruction::OpLeftShift(lhs, rhs, register) => {
        sf.write_slice_joined(" ", &[&"op<<", lhs, rhs, register])
      }
      Instruction::OpRightShift(lhs, rhs, register) => {
        sf.write_slice_joined(" ", &[&"op>>", lhs, rhs, register])
      }
      Instruction::OpRightShiftUnsigned(lhs, rhs, register) => {
        sf.write_slice_joined(" ", &[&"op>>>", lhs, rhs, register])
      }
      Instruction::TypeOf(value, register) => {
        sf.write_slice_joined(" ", &[&"typeof", value, register])
      }
      Instruction::InstanceOf(lhs, rhs, register) => {
        sf.write_slice_joined(" ", &[&"instanceof", lhs, rhs, register])
      }
      Instruction::In(lhs, rhs, register) => {
        sf.write_slice_joined(" ", &[&"in", lhs, rhs, register])
      }
      Instruction::Call(value, args, register) => {
        sf.write_slice_joined(" ", &[&"call", value, args, register])
      }
      Instruction::Apply(value, this, args, register) => {
        sf.write_slice_joined(" ", &[&"apply", value, this, args, register])
      }
      Instruction::ConstApply(value, this, args, register) => {
        sf.write_slice_joined(" ", &[&"const_apply", value, this, args, register])
      }
      Instruction::Bind(value, args, register) => {
        sf.write_slice_joined(" ", &[&"bind", value, args, register])
      }
      Instruction::Sub(lhs, rhs, register) => {
        sf.write_slice_joined(" ", &[&"sub", lhs, rhs, register])
      }
      Instruction::SubMov(subscript, value, register) => {
        sf.write_slice_joined(" ", &[&"submov", subscript, value, register])
      }
      Instruction::SubCall(this, key, args, register) => {
        sf.write_slice_joined(" ", &[&"subcall", this, key, args, register])
      }
      Instruction::Jmp(label_ref) => sf.write_slice_joined(" ", &[&"jmp", label_ref]),
      Instruction::JmpIf(value, label_ref) => {
        sf.write_slice_joined(" ", &[&"jmpif", value, label_ref])
      }
      Instruction::JmpIfNot(value, label_ref) => {
        sf.write_slice_joined(" ", &[&"jmpif_not", value, label_ref])
      }
      Instruction::UnaryPlus(value, register) => {
        sf.write_slice_joined(" ", &[&"unary+", value, register])
      }
      Instruction::UnaryMinus(value, register) => {
        sf.write_slice_joined(" ", &[&"unary-", value, register])
      }
      Instruction::New(value, args, register) => {
        sf.write_slice_joined(" ", &[&"new", value, args, register])
      }
      Instruction::Throw(value) => sf.write_slice_joined(" ", &[&"throw", value]),
      Instruction::Import(value, register) => {
        sf.write_slice_joined(" ", &[&"import", value, register])
      }
      Instruction::ImportStar(value, register) => {
        sf.write_slice_joined(" ", &[&"import*", value, register])
      }
      Instruction::SetCatch(label_ref, register) => {
        sf.write_slice_joined(" ", &[&"set_catch", label_ref, register])
      }
      Instruction::UnsetCatch => sf.write("unset_catch"),
      Instruction::ConstSubCall(a1, a2, a3, register) => {
        sf.write_slice_joined(" ", &[&"const_subcall", a1, a2, a3, register])
      }
      Instruction::RequireMutableThis => sf.write("require_mutable_this"),
      Instruction::ThisSubCall(this, key, args, register) => {
        sf.write_slice_joined(" ", &[&"this_subcall", this, key, args, register])
      }
      Instruction::Next(iterable, register) => {
        sf.write_slice_joined(" ", &[&"next", iterable, register])
      }
      Instruction::UnpackIterRes(iter_res, value_dst, done_dst) => {
        sf.write_slice_joined(" ", &[&"unpack_iter_res", iter_res, value_dst, done_dst])
      }
      Instruction::Cat(value, register) => sf.write_slice_joined(" ", &[&"cat", value, register]),
      Instruction::Yield(value, register) => {
        sf.write_slice_joined(" ", &[&"yield", value, register])
      }
      Instruction::YieldStar(value, register) => {
        sf.write_slice_joined(" ", &[&"yield*", value, register])
      }
      Instruction::Delete(obj, sub, dst) => sf.write_slice_joined(" ", &[&"delete", obj, sub, dst]),
    }
  }
}
