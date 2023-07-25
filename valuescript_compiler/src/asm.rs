use std::hash::{Hash as HashTrait, Hasher};

use num_bigint::BigInt;

use crate::{
  assembler::ValueType, expression_compiler::CompiledExpression, instruction::RegisterVisitMut,
};

pub use crate::instruction::{Instruction, InstructionFieldMut};

#[derive(Debug, Clone, Default)]
pub struct Module {
  pub export_default: Value,
  pub export_star: Object,
  pub definitions: Vec<Definition>,
}

impl Module {
  pub fn as_lines(&self) -> Vec<String> {
    let assembly_str = self.to_string();
    let assembly_lines = assembly_str.split('\n');

    assembly_lines.map(|s| s.to_string()).collect()
  }
}

impl std::fmt::Display for Module {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    if self.export_star.properties.is_empty() {
      write!(f, "export {} {}", self.export_default, self.export_star)?;
    } else {
      writeln!(f, "export {} {{", self.export_default)?;

      for (name, value) in &self.export_star.properties {
        writeln!(f, "    {}: {},", name, value)?;
      }

      write!(f, "}}")?;
    }

    for definition in &self.definitions {
      write!(f, "\n\n{}", definition)?;
    }

    Ok(())
  }
}

#[derive(Debug, Clone)]
pub struct Definition {
  pub pointer: Pointer,
  pub content: DefinitionContent,
}

impl Default for Definition {
  fn default() -> Self {
    Definition {
      pointer: Pointer {
        name: "".to_string(),
      },
      content: DefinitionContent::Value(Value::Void),
    }
  }
}

impl std::fmt::Display for Definition {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    write!(f, "{} = {}", self.pointer, self.content)
  }
}

#[derive(Debug, Clone)]
pub enum DefinitionContent {
  Function(Function),
  Class(Class),
  Value(Value),
  Lazy(Lazy),
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
      DefinitionContent::Lazy(lazy) => {
        write!(f, "{}", lazy)
      }
    }
  }
}

#[derive(Hash, PartialEq, Eq, Clone, Debug, PartialOrd, Ord)]
pub struct Pointer {
  pub name: String,
}

impl std::fmt::Display for Pointer {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    write!(f, "@{}", self.name)
  }
}

#[derive(Default, Debug, Clone)]
pub struct Function {
  pub is_generator: bool,
  pub parameters: Vec<Register>,
  pub body: Vec<FnLine>,
}

impl std::fmt::Display for Function {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    match self.is_generator {
      false => write!(f, "function(")?,
      true => write!(f, "function*(")?,
    }

    for (i, parameter) in self.parameters.iter().enumerate() {
      if i > 0 {
        write!(f, ", ")?;
      }
      write!(f, "{}", parameter)?;
    }
    writeln!(f, ") {{")?;
    for fn_line in &self.body {
      match fn_line {
        FnLine::Instruction(instruction) => writeln!(f, "    {}", instruction)?,
        FnLine::Label(label) => writeln!(f, "  {}", label)?,
        FnLine::Empty => writeln!(f)?,
        FnLine::Comment(message) => writeln!(f, "    // {}", message)?,
        FnLine::Release(reg) => writeln!(f, "    (release {})", reg)?,
      }
    }
    write!(f, "}}")
  }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Class {
  pub constructor: Value,
  pub prototype: Value,
  pub static_: Value,
}

impl std::fmt::Display for Class {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    writeln!(f, "class {{")?;

    writeln!(f, "    constructor: {},", self.constructor)?;

    write!(f, "    prototype: ")?;

    match &self.prototype {
      Value::Object(object) => {
        if object.properties.is_empty() {
          writeln!(f, "{{}},")?;
        } else {
          writeln!(f, "{{")?;
          for (name, method) in &object.properties {
            writeln!(f, "        {}: {},", name, method)?;
          }
          writeln!(f, "    }},")?;
        }
      }
      _ => {
        writeln!(f, "{},", self.prototype)?;
      }
    }

    write!(f, "    static: ")?;

    match &self.static_ {
      Value::Object(object) => {
        if object.properties.is_empty() {
          writeln!(f, "{{}},")?;
        } else {
          writeln!(f, "{{")?;
          for (name, method) in &object.properties {
            writeln!(f, "        {}: {},", name, method)?;
          }
          writeln!(f, "    }},")?;
        }
      }
      _ => {
        writeln!(f, "{},", self.prototype)?;
      }
    }

    write!(f, "}}")?;

    Ok(())
  }
}

#[derive(Hash, PartialEq, Eq, Clone, Debug, PartialOrd, Ord)]
pub struct Register {
  pub take: bool,
  pub name: String,
}

impl Register {
  pub fn return_() -> Self {
    Register {
      take: false,
      name: "return".to_string(),
    }
  }

  pub fn this() -> Self {
    Register {
      take: false,
      name: "this".to_string(),
    }
  }

  pub fn named(name: String) -> Self {
    Register { take: false, name }
  }

  pub fn ignore() -> Self {
    Register {
      take: false,
      name: "ignore".to_string(),
    }
  }

  pub fn take(&self) -> Self {
    Register {
      take: true,
      name: self.name.clone(),
    }
  }

  pub fn copy(&self) -> Self {
    Register {
      take: false,
      name: self.name.clone(),
    }
  }

  pub fn is_return(&self) -> bool {
    self.name == "return"
  }

  pub fn is_this(&self) -> bool {
    self.name == "this"
  }

  pub fn is_named(&self) -> bool {
    !matches!(self.name.as_str(), "return" | "this" | "ignore")
  }

  pub fn is_ignore(&self) -> bool {
    self.name == "ignore"
  }

  pub fn is_special(&self) -> bool {
    self.is_return() || self.is_this() || self.is_ignore()
  }

  pub fn value_type(&self) -> ValueType {
    if self.take {
      ValueType::TakeRegister
    } else {
      ValueType::Register
    }
  }
}

impl std::fmt::Display for Register {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    write!(f, "%")?;

    if self.take {
      write!(f, "!")?;
    }

    write!(f, "{}", self.name)
  }
}

#[derive(Debug, Clone)]
pub enum FnLine {
  Instruction(Instruction),
  Label(Label),
  Empty,
  Comment(String),
  Release(Register),
}

impl std::fmt::Display for FnLine {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    match self {
      FnLine::Instruction(instruction) => write!(f, "{}", instruction),
      FnLine::Label(label) => write!(f, "{}", label),
      FnLine::Empty => Ok(()),
      FnLine::Comment(message) => write!(f, "// {}", message),
      FnLine::Release(reg) => write!(f, "(release {})", reg),
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

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum Value {
  #[default]
  Void,
  Undefined,
  Null,
  Bool(bool),
  Number(Number),
  BigInt(BigInt),
  String(String),
  Array(Box<Array>),
  Object(Box<Object>),
  Class(Box<Class>),
  Register(Register),
  Pointer(Pointer),
  Builtin(Builtin),
}

#[derive(Debug, Clone)]
pub struct Number(pub f64);

impl PartialEq for Number {
  fn eq(&self, other: &Self) -> bool {
    self.0 == other.0
  }
}

impl Eq for Number {
  fn assert_receiver_is_total_eq(&self) {}
}

impl HashTrait for Number {
  fn hash<H: Hasher>(&self, state: &mut H) {
    state.write_u64(self.0.to_bits());
  }
}

impl Value {
  pub fn to_ce(self) -> CompiledExpression {
    CompiledExpression::new(self, vec![])
  }

  pub fn visit_values_mut<F>(&mut self, visit: &mut F)
  where
    F: FnMut(&mut Value),
  {
    visit(self);

    match self {
      Value::Array(array) => {
        for item in &mut array.values {
          item.visit_values_mut(visit);
        }
      }
      Value::Object(object) => {
        for (k, v) in &mut object.properties {
          k.visit_values_mut(visit);
          v.visit_values_mut(visit);
        }
      }
      Value::Class(class) => {
        class.constructor.visit_values_mut(visit);
        class.prototype.visit_values_mut(visit);
        class.static_.visit_values_mut(visit);
      }
      Value::Void => {}
      Value::Undefined => {}
      Value::Null => {}
      Value::Bool(..) => {}
      Value::Number(..) => {}
      Value::BigInt(..) => {}
      Value::String(..) => {}
      Value::Register(..) => {}
      Value::Pointer(..) => {}
      Value::Builtin(..) => {}
    }
  }

  pub fn visit_registers_mut_rev<F>(&mut self, visit: &mut F)
  where
    F: FnMut(RegisterVisitMut),
  {
    match self {
      Value::Array(array) => {
        for item in &mut array.values.iter_mut().rev() {
          item.visit_registers_mut_rev(visit);
        }
      }
      Value::Object(object) => {
        for (k, v) in &mut object.properties.iter_mut().rev() {
          v.visit_registers_mut_rev(visit);
          k.visit_registers_mut_rev(visit);
        }
      }
      Value::Class(class) => {
        class.constructor.visit_registers_mut_rev(visit);
        class.prototype.visit_registers_mut_rev(visit);
        class.static_.visit_registers_mut_rev(visit);
      }
      Value::Void => {}
      Value::Undefined => {}
      Value::Null => {}
      Value::Bool(..) => {}
      Value::Number(..) => {}
      Value::BigInt(..) => {}
      Value::String(..) => {}
      Value::Register(register) => {
        visit(RegisterVisitMut::read(register));
      }
      Value::Pointer(..) => {}
      Value::Builtin(..) => {}
    }
  }
}

impl std::fmt::Display for Value {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Value::Void => write!(f, "void"),
      Value::Undefined => write!(f, "undefined"),
      Value::Null => write!(f, "null"),
      Value::Bool(value) => write!(f, "{}", value),
      Value::Number(Number(value)) => {
        if value.is_infinite() {
          if value.is_sign_positive() {
            write!(f, "Infinity")
          } else {
            write!(f, "-Infinity")
          }
        } else {
          write!(f, "{}", value)
        }
      }
      Value::BigInt(value) => write!(f, "{}n", value),
      Value::String(value) => write!(
        f,
        "{}",
        serde_json::to_string(&value).expect("Failed json serialization")
      ),
      Value::Array(value) => write!(f, "{}", value),
      Value::Object(value) => write!(f, "{}", value),
      Value::Class(class) => write!(f, "{}", class),
      Value::Register(value) => write!(f, "{}", value),
      Value::Pointer(value) => write!(f, "{}", value),
      Value::Builtin(value) => write!(f, "{}", value),
    }
  }
}

#[derive(Debug, Clone)]
pub struct Lazy {
  pub body: Vec<FnLine>,
}

impl std::fmt::Display for Lazy {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    writeln!(f, "lazy {{")?;

    for fn_line in &self.body {
      match fn_line {
        FnLine::Instruction(instruction) => writeln!(f, "    {}", instruction)?,
        FnLine::Label(label) => writeln!(f, "  {}", label)?,
        FnLine::Empty => writeln!(f)?,
        FnLine::Comment(message) => writeln!(f, "    // {}", message)?,
        FnLine::Release(reg) => writeln!(f, "    (release {})", reg)?,
      }
    }

    write!(f, "}}")
  }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Builtin {
  pub name: String,
}

impl std::fmt::Display for Builtin {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "${}", self.name)
  }
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Hash)]
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

#[derive(Default, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Object {
  pub properties: Vec<(Value, Value)>,
}

impl std::fmt::Display for Object {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    if self.properties.is_empty() {
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

impl Object {
  pub fn try_resolve_key(&self, key: &String) -> Option<&Value> {
    let mut result: Option<&Value> = None;

    for (k, v) in &self.properties {
      if let Value::String(k) = k {
        if k == key {
          result = Some(v);
        }
      } else {
        // If the key is not a string, it's possible that the result we found earlier is overwritten
        // here, so we have to set back to None.
        result = None;
      }
    }

    result
  }
}
