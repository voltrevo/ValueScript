#[derive(Debug, Clone, PartialEq)]
pub enum InstructionByte {
  End = 0x00,
  Mov = 0x01,
  OpInc = 0x02,
  OpDec = 0x03,
  OpPlus = 0x04,
  OpMinus = 0x05,
  OpMul = 0x06,
  OpDiv = 0x07,
  OpMod = 0x08,
  OpExp = 0x09,
  OpEq = 0x0a,
  OpNe = 0x0b,
  OpTripleEq = 0x0c,
  OpTripleNe = 0x0d,
  OpAnd = 0x0e,
  OpOr = 0x0f,
  OpNot = 0x10,
  OpLess = 0x11,
  OpLessEq = 0x12,
  OpGreater = 0x13,
  OpGreaterEq = 0x14,
  OpNullishCoalesce = 0x15,
  OpOptionalChain = 0x16,
  OpBitAnd = 0x17,
  OpBitOr = 0x18,
  OpBitNot = 0x19,
  OpBitXor = 0x1a,
  OpLeftShift = 0x1b,
  OpRightShift = 0x1c,
  OpRightShiftUnsigned = 0x1d,
  TypeOf = 0x1e,
  InstanceOf = 0x1f,
  In = 0x20,
  Call = 0x21,
  Apply = 0x22,
  Bind = 0x23,
  Sub = 0x24,
  SubMov = 0x25,
  SubCall = 0x26,
  Jmp = 0x27,
  JmpIf = 0x28,
  UnaryPlus = 0x29,
  UnaryMinus = 0x2a,
  New = 0x2b,
  Throw = 0x2c,
  Import = 0x2d,
  ImportStar = 0x2e,
  SetCatch = 0x2f,
  UnsetCatch = 0x30,
  ConstSubCall = 0x31,
}

impl InstructionByte {
  pub fn from_byte(byte: u8) -> InstructionByte {
    use InstructionByte::*;

    return match byte {
      0x00 => End,
      0x01 => Mov,
      0x02 => OpInc,
      0x03 => OpDec,
      0x04 => OpPlus,
      0x05 => OpMinus,
      0x06 => OpMul,
      0x07 => OpDiv,
      0x08 => OpMod,
      0x09 => OpExp,
      0x0a => OpEq,
      0x0b => OpNe,
      0x0c => OpTripleEq,
      0x0d => OpTripleNe,
      0x0e => OpAnd,
      0x0f => OpOr,
      0x10 => OpNot,
      0x11 => OpLess,
      0x12 => OpLessEq,
      0x13 => OpGreater,
      0x14 => OpGreaterEq,
      0x15 => OpNullishCoalesce,
      0x16 => OpOptionalChain,
      0x17 => OpBitAnd,
      0x18 => OpBitOr,
      0x19 => OpBitNot,
      0x1a => OpBitXor,
      0x1b => OpLeftShift,
      0x1c => OpRightShift,
      0x1d => OpRightShiftUnsigned,
      0x1e => TypeOf,
      0x1f => InstanceOf,
      0x20 => In,
      0x21 => Call,
      0x22 => Apply,
      0x23 => Bind,
      0x24 => Sub,
      0x25 => SubMov,
      0x26 => SubCall,
      0x27 => Jmp,
      0x28 => JmpIf,
      0x29 => UnaryPlus,
      0x2a => UnaryMinus,
      0x2b => New,
      0x2c => Throw,
      0x2d => Import,
      0x2e => ImportStar,
      0x2f => SetCatch,
      0x30 => UnsetCatch,
      0x31 => ConstSubCall,

      _ => panic!("Unrecognized instruction: {}", byte),
    };
  }
}
