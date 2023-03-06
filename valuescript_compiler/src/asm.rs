#[derive(Debug)]
pub struct Module {
  pub definitions: Vec<Definition>,
}

impl std::fmt::Display for Module {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    for definition in &self.definitions {
      write!(f, "{}\n", definition)?;
    }

    return Ok(());
  }
}

#[derive(Debug)]
pub struct Definition {
  pub pointer: Pointer,
  pub content: DefinitionContent,
}

impl std::fmt::Display for Definition {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    write!(f, "{} = {}", self.pointer, self.content)
  }
}

#[derive(Debug)]
pub enum DefinitionContent {
  Function(Function),
  Class(Class),
  Value(Value),
}

impl std::fmt::Display for DefinitionContent {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    match self {
      DefinitionContent::Function(function) => {
        write!(f, "{}", function)
      }
      DefinitionContent::Class(class) => {
        write!(f, "{}", class)
      }
      DefinitionContent::Value(value) => {
        write!(f, "{}", value)
      }
    }
  }
}

#[derive(Hash, PartialEq, Eq, Clone, Debug)]
pub struct Pointer {
  pub name: String,
}

impl std::fmt::Display for Pointer {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    write!(f, "@{}", self.name)
  }
}

#[derive(Default, Debug)]
pub struct Function {
  pub parameters: Vec<Register>,
  pub body: Vec<InstructionOrLabel>,
}

impl std::fmt::Display for Function {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    write!(f, "function(")?;
    for (i, parameter) in self.parameters.iter().enumerate() {
      if i > 0 {
        write!(f, ", ")?;
      }
      write!(f, "{}", parameter)?;
    }
    write!(f, ") {{\n")?;
    for instruction_or_label in &self.body {
      match instruction_or_label {
        InstructionOrLabel::Instruction(instruction) => {
          write!(f, "  {}\n", instruction)?;
        }
        InstructionOrLabel::Label(label) => {
          write!(f, "{}\n", label)?;
        }
      }
    }
    write!(f, "}}")
  }
}

#[derive(Debug)]
pub struct Class {
  pub constructor: Value,
  pub methods: Value,
}

impl std::fmt::Display for Class {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    write!(f, "class({}, ", self.constructor)?;

    match &self.methods {
      Value::Object(object) => {
        write!(f, "{{\n")?;
        for (name, method) in &object.properties {
          write!(f, "  {}: {}\n", name, method)?;
        }
        write!(f, "}})\n")?;
      }
      _ => {
        write!(f, "{})\n", self.methods)?;
      }
    }

    return Ok(());
  }
}

#[derive(Hash, PartialEq, Eq, Clone, Debug)]
pub enum Register {
  Return,
  This,
  Named(String),
  Ignore,
}

impl Register {
  pub fn as_name(&self) -> String {
    match self {
      Register::Return => "return".to_string(),
      Register::This => "this".to_string(),
      Register::Named(name) => name.clone(),
      Register::Ignore => "ignore".to_string(),
    }
  }
}

impl std::fmt::Display for Register {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    match self {
      Register::Return => write!(f, "%return"),
      Register::This => write!(f, "%this"),
      Register::Named(name) => write!(f, "%{}", name),
      Register::Ignore => write!(f, "%ignore"),
    }
  }
}

#[derive(Debug)]
pub enum InstructionOrLabel {
  Instruction(Instruction),
  Label(Label),
}

impl std::fmt::Display for InstructionOrLabel {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    match self {
      InstructionOrLabel::Instruction(instruction) => {
        write!(f, "{}", instruction)
      }
      InstructionOrLabel::Label(label) => {
        write!(f, "{}", label)
      }
    }
  }
}

#[derive(Debug, Clone)]
pub struct Label {
  pub name: String,
}

impl Label {
  pub fn ref_(&self) -> LabelRef {
    LabelRef {
      name: self.name.clone(),
    }
  }
}

impl std::fmt::Display for Label {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    write!(f, "{}:", self.name)
  }
}

#[derive(Hash, PartialEq, Eq, Clone, Debug)]
pub struct LabelRef {
  pub name: String,
}

impl std::fmt::Display for LabelRef {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    write!(f, ":{}", self.name)
  }
}

#[derive(Debug)]
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
  Jmp(LabelRef),
  JmpIf(Value, LabelRef),
  UnaryPlus(Value, Register),
  UnaryMinus(Value, Register),
  New(Value, Value, Register),
}

impl std::fmt::Display for Instruction {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    match self {
      Instruction::End => write!(f, "end"),
      Instruction::Mov(value, register) => {
        write!(f, "mov {} {}", value, register)
      }
      Instruction::OpInc(register) => write!(f, "op++ {}", register),
      Instruction::OpDec(register) => write!(f, "op-- {}", register),
      Instruction::OpPlus(lhs, rhs, register) => {
        write!(f, "op+ {} {} {}", lhs, rhs, register)
      }
      Instruction::OpMinus(lhs, rhs, register) => {
        write!(f, "op- {} {} {}", lhs, rhs, register)
      }
      Instruction::OpMul(lhs, rhs, register) => {
        write!(f, "op* {} {} {}", lhs, rhs, register)
      }
      Instruction::OpDiv(lhs, rhs, register) => {
        write!(f, "op/ {} {} {}", lhs, rhs, register)
      }
      Instruction::OpMod(lhs, rhs, register) => {
        write!(f, "op% {} {} {}", lhs, rhs, register)
      }
      Instruction::OpExp(lhs, rhs, register) => {
        write!(f, "op** {} {} {}", lhs, rhs, register)
      }
      Instruction::OpEq(lhs, rhs, register) => {
        write!(f, "op== {} {} {}", lhs, rhs, register)
      }
      Instruction::OpNe(lhs, rhs, register) => {
        write!(f, "op!= {} {} {}", lhs, rhs, register)
      }
      Instruction::OpTripleEq(lhs, rhs, register) => {
        write!(f, "op=== {} {} {}", lhs, rhs, register)
      }
      Instruction::OpTripleNe(lhs, rhs, register) => {
        write!(f, "op!== {} {} {}", lhs, rhs, register)
      }
      Instruction::OpAnd(lhs, rhs, register) => {
        write!(f, "op&& {} {} {}", lhs, rhs, register)
      }
      Instruction::OpOr(lhs, rhs, register) => {
        write!(f, "op|| {} {} {}", lhs, rhs, register)
      }
      Instruction::OpNot(value, register) => {
        write!(f, "op! {} {}", value, register)
      }
      Instruction::OpLess(lhs, rhs, register) => {
        write!(f, "op< {} {} {}", lhs, rhs, register)
      }
      Instruction::OpLessEq(lhs, rhs, register) => {
        write!(f, "op<= {} {} {}", lhs, rhs, register)
      }
      Instruction::OpGreater(lhs, rhs, register) => {
        write!(f, "op> {} {} {}", lhs, rhs, register)
      }
      Instruction::OpGreaterEq(lhs, rhs, register) => {
        write!(f, "op>= {} {} {}", lhs, rhs, register)
      }
      Instruction::OpNullishCoalesce(lhs, rhs, register) => {
        write!(f, "op?? {} {} {}", lhs, rhs, register)
      }
      Instruction::OpOptionalChain(lhs, rhs, register) => {
        write!(f, "op?. {} {} {}", lhs, rhs, register)
      }
      Instruction::OpBitAnd(lhs, rhs, register) => {
        write!(f, "op& {} {} {}", lhs, rhs, register)
      }
      Instruction::OpBitOr(lhs, rhs, register) => {
        write!(f, "op| {} {} {}", lhs, rhs, register)
      }
      Instruction::OpBitNot(value, register) => {
        write!(f, "op~ {} {}", value, register)
      }
      Instruction::OpBitXor(lhs, rhs, register) => {
        write!(f, "op^ {} {} {}", lhs, rhs, register)
      }
      Instruction::OpLeftShift(lhs, rhs, register) => {
        write!(f, "op<< {} {} {}", lhs, rhs, register)
      }
      Instruction::OpRightShift(lhs, rhs, register) => {
        write!(f, "op>> {} {} {}", lhs, rhs, register)
      }
      Instruction::OpRightShiftUnsigned(lhs, rhs, register) => {
        write!(f, "op>>> {} {} {}", lhs, rhs, register)
      }
      Instruction::TypeOf(value, register) => {
        write!(f, "typeof {} {}", value, register)
      }
      Instruction::InstanceOf(lhs, rhs, register) => {
        write!(f, "instanceof {} {} {}", lhs, rhs, register)
      }
      Instruction::In(lhs, rhs, register) => {
        write!(f, "in {} {} {}", lhs, rhs, register)
      }
      Instruction::Call(value, args, register) => {
        write!(f, "call {} {} {}", value, args, register)
      }
      Instruction::Apply(value, this, args, register) => {
        write!(f, "apply {} {} {} {}", value, this, args, register)
      }
      Instruction::Bind(value, args, register) => {
        write!(f, "bind {} {} {}", value, args, register)
      }
      Instruction::Sub(lhs, rhs, register) => {
        write!(f, "sub {} {} {}", lhs, rhs, register)
      }
      Instruction::SubMov(subscript, value, register) => {
        write!(f, "submov {} {} {}", subscript, value, register)
      }
      Instruction::SubCall(obj, subscript, args, register) => {
        write!(f, "subcall {} {} {} {}", obj, subscript, args, register)
      }
      Instruction::Jmp(label_ref) => write!(f, "jmp {}", label_ref),
      Instruction::JmpIf(value, label_ref) => {
        write!(f, "jmpif {} {}", value, label_ref)
      }
      Instruction::UnaryPlus(value, register) => {
        write!(f, "unary+ {} {}", value, register)
      }
      Instruction::UnaryMinus(value, register) => {
        write!(f, "unary- {} {}", value, register)
      }
      Instruction::New(value, args, register) => {
        write!(f, "new {} {} {}", value, args, register)
      }
    }
  }
}

impl Instruction {
  pub fn byte(&self) -> u8 {
    use Instruction::*;

    // TODO: Define this in one place only
    match self {
      End => 0x00,
      Mov(..) => 0x01,
      OpInc(..) => 0x02,
      OpDec(..) => 0x03,
      OpPlus(..) => 0x04,
      OpMinus(..) => 0x05,
      OpMul(..) => 0x06,
      OpDiv(..) => 0x07,
      OpMod(..) => 0x08,
      OpExp(..) => 0x09,
      OpEq(..) => 0x0a,
      OpNe(..) => 0x0b,
      OpTripleEq(..) => 0x0c,
      OpTripleNe(..) => 0x0d,
      OpAnd(..) => 0x0e,
      OpOr(..) => 0x0f,
      OpNot(..) => 0x10,
      OpLess(..) => 0x11,
      OpLessEq(..) => 0x12,
      OpGreater(..) => 0x13,
      OpGreaterEq(..) => 0x14,
      OpNullishCoalesce(..) => 0x15,
      OpOptionalChain(..) => 0x16,
      OpBitAnd(..) => 0x17,
      OpBitOr(..) => 0x18,
      OpBitNot(..) => 0x19,
      OpBitXor(..) => 0x1a,
      OpLeftShift(..) => 0x1b,
      OpRightShift(..) => 0x1c,
      OpRightShiftUnsigned(..) => 0x1d,
      TypeOf(..) => 0x1e,
      InstanceOf(..) => 0x1f,
      In(..) => 0x20,
      Call(..) => 0x21,
      Apply(..) => 0x22,
      Bind(..) => 0x23,
      Sub(..) => 0x24,
      SubMov(..) => 0x25,
      SubCall(..) => 0x26,
      Jmp(..) => 0x27,
      JmpIf(..) => 0x28,
      UnaryPlus(..) => 0x29,
      UnaryMinus(..) => 0x2a,
      New(..) => 0x2b,
    }
  }
}

#[derive(Debug, Clone)]
pub enum Value {
  Void,
  Undefined,
  Null,
  Bool(bool),
  Number(f64),
  String(String),
  Array(Box<Array>),
  Object(Box<Object>),
  Register(Register),
  Pointer(Pointer),
  Builtin(Builtin),
}

impl std::fmt::Display for Value {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Value::Void => write!(f, "void"),
      Value::Undefined => write!(f, "undefined"),
      Value::Null => write!(f, "null"),
      Value::Bool(value) => write!(f, "{}", value),
      Value::Number(value) => write!(f, "{}", value),
      Value::String(value) => write!(
        f,
        "{}",
        serde_json::to_string(&value).expect("Failed json serialization")
      ),
      Value::Array(value) => write!(f, "{}", value),
      Value::Object(value) => write!(f, "{}", value),
      Value::Register(value) => write!(f, "{}", value),
      Value::Pointer(value) => write!(f, "{}", value),
      Value::Builtin(value) => write!(f, "{}", value),
    }
  }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Builtin {
  pub name: String,
}

impl std::fmt::Display for Builtin {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "${}", self.name)
  }
}

#[derive(Default, Debug, Clone)]
pub struct Array {
  pub values: Vec<Value>,
}

impl std::fmt::Display for Array {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "[")?;
    for (i, value) in self.values.iter().enumerate() {
      if i > 0 {
        write!(f, ", ")?;
      }
      write!(f, "{}", value)?;
    }
    write!(f, "]")
  }
}

#[derive(Default, Debug, Clone)]
pub struct Object {
  pub properties: Vec<(Value, Value)>,
}

impl std::fmt::Display for Object {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    if self.properties.len() == 0 {
      return write!(f, "{{}}");
    }

    write!(f, "{{ ")?;
    for (i, (key, value)) in self.properties.iter().enumerate() {
      if i > 0 {
        write!(f, ", ")?;
      }
      write!(f, "{}: {}", key, value)?;
    }
    write!(f, " }}")
  }
}
