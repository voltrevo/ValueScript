// pub fn foo() -> Assembly {}

struct Assembly {
  definitions: Vec<Definition>,
}

struct Definition {
  ref_: DefinitionRef,
  content: DefinitionContent,
}

enum DefinitionContent {
  Function(Function),
  Value(Value),
}

struct DefinitionRef {
  name: String,
}

struct Function {
  parameters: Vec<Register>,
  body: Vec<InstructionOrLabel>,
}

enum Register {
  Return,
  This,
  Named(String),
}

enum InstructionOrLabel {
  Instruction(Instruction),
  Label(Label),
}

struct Label {
  name: String,
}

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
  Apply(Value, Value, Value, Register),
  Bind(Value, Value, Register),
  Sub(Value, Value, Register),
  SubMov(Value, Value, Register),
  SubCall(Value, Value, Value, Register),
  Jmp(Label),
  JmpIf(Value, Label),
  UnaryPlus(Value, Register),
  UnaryMinus(Value, Register),
  New(Value, Value, Register),
}

enum Value {
  Undefined,
  Null,
  Boolean(bool),
  Number(f64),
  String(String),
  Array(Box<Array>),
  Object(Box<Object>),
  Register(Register),
  DefinitionRef(DefinitionRef),
  Label(Label),
}

struct Array {
  values: Vec<Value>,
}

struct Object {
  properties: Vec<(String, Value)>,
}
