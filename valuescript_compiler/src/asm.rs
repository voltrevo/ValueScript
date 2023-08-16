use std::{
  collections::HashMap,
  fmt::Write,
  hash::{Hash as HashTrait, Hasher},
};

use num_bigint::BigInt;

use crate::{
  assembler::ValueType, expression_compiler::CompiledExpression, instruction::RegisterVisitMut,
};

pub use crate::instruction::{Instruction, InstructionFieldMut};

pub struct StructuredFormatter<'a, 'b> {
  f: &'a mut std::fmt::Formatter<'b>,
  indent_level: usize,
  indent_str: &'a str,
  indent_str_short: &'a str,
}

impl<'a, 'b> StructuredFormatter<'a, 'b> {
  pub fn nest<F: FnMut(&mut StructuredFormatter<'_, '_>) -> std::fmt::Result>(
    &mut self,
    mut f: F,
  ) -> std::fmt::Result {
    let mut nested = StructuredFormatter {
      f: self.f,
      indent_level: self.indent_level + 1,
      indent_str: self.indent_str,
      indent_str_short: self.indent_str_short,
    };

    f(&mut nested)
  }

  pub fn indent(&mut self) -> std::fmt::Result {
    for _ in 0..self.indent_level {
      self.f.write_str(self.indent_str)?;
    }

    Ok(())
  }

  pub fn write<T>(&mut self, data: &T) -> std::fmt::Result
  where
    T: StructuredFormattable + ?Sized,
  {
    data.structured_fmt(self)
  }

  pub fn newline(&mut self) -> std::fmt::Result {
    self.f.write_char('\n')?;
    self.indent()
  }

  pub fn newline_short(&mut self) -> std::fmt::Result {
    self.f.write_char('\n')?;

    for _ in 0..(self.indent_level - 1) {
      self.f.write_str(self.indent_str)?;
    }

    self.f.write_str(self.indent_str_short)
  }

  pub fn write_slice_joined(
    &mut self,
    sep: &str,
    data: &[&dyn StructuredFormattable],
  ) -> std::fmt::Result {
    let mut iter = data.iter();

    if let Some(first) = iter.next() {
      self.write(*first)?;

      for item in iter {
        self.write(&sep)?;
        self.write(*item)?;
      }
    }

    Ok(())
  }

  pub fn write_slice(&mut self, data: &[&dyn StructuredFormattable]) -> std::fmt::Result {
    for item in data {
      self.write(*item)?;
    }

    Ok(())
  }

  pub fn write_line(&mut self, data: &[&dyn StructuredFormattable]) -> std::fmt::Result {
    self.newline()?;
    self.write_slice(data)
  }
}

pub trait StructuredFormattable {
  fn structured_fmt(&self, sf: &mut StructuredFormatter<'_, '_>) -> std::fmt::Result;
}

impl StructuredFormattable for str {
  fn structured_fmt(&self, sf: &mut StructuredFormatter<'_, '_>) -> std::fmt::Result {
    sf.f.write_str(self)
  }
}

impl StructuredFormattable for &str {
  fn structured_fmt(&self, sf: &mut StructuredFormatter<'_, '_>) -> std::fmt::Result {
    sf.f.write_str(self)
  }
}

impl StructuredFormattable for String {
  fn structured_fmt(&self, sf: &mut StructuredFormatter<'_, '_>) -> std::fmt::Result {
    sf.f.write_str(self)
  }
}

pub struct Structured<'a, T>(pub &'a T);

impl<'a, T: StructuredFormattable> std::fmt::Display for Structured<'a, T> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    StructuredFormatter {
      f,
      indent_level: 0,
      indent_str: "    ",
      indent_str_short: "  ",
    }
    .write(self.0)
  }
}

#[derive(Debug, Clone, Default)]
pub struct Module {
  pub export_default: Value,
  pub export_star: ExportStar,
  pub definitions: Vec<Definition>,
}

#[derive(Debug, Clone, Default)]
pub struct ExportStar {
  pub includes: Vec<Pointer>,
  pub local: Object,
}

impl StructuredFormattable for ExportStar {
  fn structured_fmt(&self, sf: &mut StructuredFormatter<'_, '_>) -> std::fmt::Result {
    if self.local.properties.is_empty() && self.includes.is_empty() {
      return sf.f.write_str("{}");
    }

    sf.write("{")?;

    sf.nest(|sf| {
      for p in &self.includes {
        sf.write_line(&[&"include ", p, &","])?;
      }

      for (name, value) in &self.local.properties {
        sf.write_line(&[name, &": ", value, &","])?;
      }

      Ok(())
    })?;

    sf.write("}")
  }
}

impl Module {
  pub fn as_lines(&self) -> Vec<String> {
    let assembly_str = format!("{}", Structured(self));
    let assembly_lines = assembly_str.split('\n');

    assembly_lines.map(|s| s.to_string()).collect()
  }

  pub fn ptr_to_index(&self) -> HashMap<Pointer, usize> {
    let mut res = HashMap::<Pointer, usize>::new();

    for (i, defn) in self.definitions.iter().enumerate() {
      res.insert(defn.pointer.clone(), i);
    }

    res
  }

  pub fn get<'a>(
    &'a self,
    ptr_to_index: &HashMap<Pointer, usize>,
    ptr: &Pointer,
  ) -> &'a DefinitionContent {
    &self.definitions[*ptr_to_index.get(ptr).unwrap()].content
  }
}

impl StructuredFormattable for Module {
  fn structured_fmt(&self, sf: &mut StructuredFormatter<'_, '_>) -> std::fmt::Result {
    sf.write_slice_joined(
      " ",
      &[
        &"export",
        &MultilineValue(&self.export_default),
        &self.export_star,
      ],
    )?;

    for definition in &self.definitions {
      sf.newline()?;
      sf.newline()?;
      sf.write(definition)?;
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

impl StructuredFormattable for Definition {
  fn structured_fmt(&self, sf: &mut StructuredFormatter<'_, '_>) -> std::fmt::Result {
    sf.write_slice_joined(" ", &[&self.pointer, &"=", &self.content])
  }
}

#[derive(Debug, Clone)]
pub enum DefinitionContent {
  Function(Function),
  Meta(Meta),
  Value(Value),
  Lazy(Lazy),
}

impl StructuredFormattable for DefinitionContent {
  fn structured_fmt(&self, sf: &mut StructuredFormatter<'_, '_>) -> std::fmt::Result {
    match self {
      DefinitionContent::Function(function) => sf.write(function),
      DefinitionContent::Meta(meta) => sf.write(meta),
      DefinitionContent::Value(value) => sf.write(value),
      DefinitionContent::Lazy(lazy) => sf.write(lazy),
    }
  }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct Meta {
  pub name: String,
  pub content_hashable: ContentHashable,
}

impl StructuredFormattable for Meta {
  fn structured_fmt(&self, sf: &mut StructuredFormatter<'_, '_>) -> std::fmt::Result {
    sf.write("meta {")?;

    sf.nest(|sf| {
      sf.write_line(&[
        &"name: ",
        &serde_json::to_string(&self.name).expect("Failed json serialization"),
        &",",
      ])?;

      match &self.content_hashable {
        ContentHashable::Empty => {}
        ContentHashable::Src(src_hash, deps) => {
          sf.write_line(&[&"srcHash: ", src_hash, &","])?;
          sf.write_line(&[
            &"deps: ",
            &Array {
              values: deps.clone(),
            },
            &",",
          ])?;
        }
        ContentHashable::Content(content_hash) => {
          sf.write_line(&[&"contentHash: ", content_hash, &","])?;
        }
      }

      Ok(())
    })?;

    sf.write_line(&[&"}"])
  }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct Hash(pub [u8; 32]);

impl StructuredFormattable for Hash {
  fn structured_fmt(&self, sf: &mut StructuredFormatter<'_, '_>) -> std::fmt::Result {
    sf.write("#")?;

    for b in self.0 {
      sf.write(&format!("{:02x}", b))?;
    }

    Ok(())
  }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub enum ContentHashable {
  #[default]
  Empty,
  Src(Hash, Vec<Value>),
  Content(Hash),
}

#[derive(Hash, PartialEq, Eq, Clone, Debug, PartialOrd, Ord)]
pub struct Pointer {
  pub name: String,
}

impl StructuredFormattable for Pointer {
  fn structured_fmt(&self, sf: &mut StructuredFormatter<'_, '_>) -> std::fmt::Result {
    sf.write("@")?;
    sf.write(&self.name)
  }
}

#[derive(Default, Debug, Clone)]
pub struct Function {
  pub is_generator: bool,
  pub meta: Option<Pointer>,
  pub parameters: Vec<Register>,
  pub body: Vec<FnLine>,
}

impl StructuredFormattable for Function {
  fn structured_fmt(&self, sf: &mut StructuredFormatter<'_, '_>) -> std::fmt::Result {
    let meta_str = match &self.meta {
      None => "".to_string(),
      Some(p) => format!(" {}", Structured(p)),
    };

    match self.is_generator {
      false => sf.write(&format!("function{}(", meta_str))?,
      true => sf.write(&format!("function*{}(", meta_str))?,
    }

    for (i, parameter) in self.parameters.iter().enumerate() {
      if i > 0 {
        sf.write(", ")?;
      }
      sf.write(parameter)?;
    }
    sf.write(") {")?;

    sf.nest(|sf| {
      for fn_line in &self.body {
        match fn_line {
          FnLine::Label(..) => sf.newline_short()?,
          _ => sf.newline()?,
        };

        sf.write(fn_line)?;
      }

      Ok(())
    })?;

    sf.newline()?;
    sf.write("}")
  }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Class {
  pub meta: Meta,
  pub constructor: Value,
  pub prototype: Value,
  pub static_: Value,
}

impl StructuredFormattable for Class {
  fn structured_fmt(&self, sf: &mut StructuredFormatter<'_, '_>) -> std::fmt::Result {
    sf.write("class {")?;

    sf.nest(|sf| {
      sf.write_line(&[&"meta: ", &self.meta, &","])?;
      sf.write_line(&[&"constructor: ", &self.constructor, &","])?;
      sf.write_line(&[&"prototype: ", &MultilineValue(&self.prototype), &","])?;
      sf.write_line(&[&"static: ", &MultilineValue(&self.static_), &","])?;

      Ok(())
    })?;

    sf.newline()?;
    sf.write("}")
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

impl StructuredFormattable for Register {
  fn structured_fmt(&self, sf: &mut StructuredFormatter<'_, '_>) -> std::fmt::Result {
    sf.write("%")?;

    if self.take {
      sf.write("!")?;
    }

    sf.write(&self.name)
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

impl StructuredFormattable for FnLine {
  fn structured_fmt(&self, sf: &mut StructuredFormatter<'_, '_>) -> std::fmt::Result {
    match self {
      FnLine::Instruction(instruction) => sf.write(instruction),
      FnLine::Label(label) => sf.write(label),
      FnLine::Empty => Ok(()),
      FnLine::Comment(message) => sf.write(&format!("// {}", message)),
      FnLine::Release(reg) => sf.write(&format!("(release {})", Structured(reg))),
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

impl StructuredFormattable for Label {
  fn structured_fmt(&self, sf: &mut StructuredFormatter<'_, '_>) -> std::fmt::Result {
    sf.write_slice(&[&self.name, &":"])
  }
}

#[derive(Hash, PartialEq, Eq, Clone, Debug)]
pub struct LabelRef {
  pub name: String,
}

impl StructuredFormattable for LabelRef {
  fn structured_fmt(&self, sf: &mut StructuredFormatter<'_, '_>) -> std::fmt::Result {
    sf.write_slice(&[&":", &self.name])
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
    if self.0.is_nan() {
      // Note: We do this so that the compiler can track dependencies, so it just wants to know
      // the identities of things. In general it's not a good idea to break the well-established
      // practice that NaN !== NaN.
      other.0.is_nan()
    } else {
      self.0 == other.0
    }
  }
}

impl Eq for Number {
  fn assert_receiver_is_total_eq(&self) {}
}

impl HashTrait for Number {
  fn hash<H: Hasher>(&self, state: &mut H) {
    state.write_u64(match self.0.is_nan() {
      true => f64::NAN.to_bits(),
      false => self.0.to_bits(),
    });
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

impl StructuredFormattable for Value {
  fn structured_fmt(&self, sf: &mut StructuredFormatter<'_, '_>) -> std::fmt::Result {
    match self {
      Value::Void => sf.write("void"),
      Value::Undefined => sf.write("undefined"),
      Value::Null => sf.write("null"),
      Value::Bool(value) => sf.write(&value.to_string()),
      Value::Number(Number(value)) => {
        if value.is_infinite() {
          if value.is_sign_positive() {
            sf.write("Infinity")
          } else {
            sf.write("-Infinity")
          }
        } else {
          sf.write(&value.to_string())
        }
      }
      Value::BigInt(value) => sf.write_slice(&[&value.to_string(), &"n"]),
      Value::String(value) => {
        sf.write(&serde_json::to_string(&value).expect("Failed json serialization"))
      }
      Value::Array(value) => sf.write(&**value),
      Value::Object(value) => sf.write(&**value),
      Value::Class(value) => sf.write(&**value),
      Value::Register(value) => sf.write(value),
      Value::Pointer(value) => sf.write(value),
      Value::Builtin(value) => sf.write(value),
    }
  }
}

struct MultilineValue<'a>(&'a Value);

impl<'a> StructuredFormattable for MultilineValue<'a> {
  fn structured_fmt(&self, sf: &mut StructuredFormatter<'_, '_>) -> std::fmt::Result {
    match self.0 {
      Value::Array(array) => {
        if array.values.is_empty() {
          return sf.write("[]");
        }

        sf.write("[")?;

        sf.nest(|sf| {
          for value in &array.values {
            sf.newline()?;
            sf.write(value)?;
            sf.write(",")?;
          }

          Ok(())
        })?;

        sf.newline()?;
        sf.write("]")
      }
      Value::Object(object) => {
        if object.properties.is_empty() {
          return sf.write("{}");
        }

        sf.write("{")?;

        sf.nest(|sf| {
          for (key, value) in &object.properties {
            sf.newline()?;
            sf.write(key)?;
            sf.write(": ")?;
            sf.write(value)?;
            sf.write(",")?;
          }

          Ok(())
        })?;

        sf.newline()?;
        sf.write("}")
      }
      _ => sf.write(self.0),
    }
  }
}

#[derive(Debug, Clone)]
pub struct Lazy {
  pub body: Vec<FnLine>,
}

impl StructuredFormattable for Lazy {
  fn structured_fmt(&self, sf: &mut StructuredFormatter<'_, '_>) -> std::fmt::Result {
    sf.write("lazy {")?;

    sf.nest(|sf| {
      for fn_line in &self.body {
        sf.newline()?;
        sf.write(fn_line)?;
      }

      Ok(())
    })?;

    sf.newline()?;
    sf.write("}")
  }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Builtin {
  pub name: String,
}

impl StructuredFormattable for Builtin {
  fn structured_fmt(&self, sf: &mut StructuredFormatter<'_, '_>) -> std::fmt::Result {
    sf.write_slice(&[&"$", &self.name])
  }
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Array {
  pub values: Vec<Value>,
}

impl StructuredFormattable for Array {
  fn structured_fmt(&self, sf: &mut StructuredFormatter<'_, '_>) -> std::fmt::Result {
    sf.write("[")?;

    for (i, value) in self.values.iter().enumerate() {
      if i > 0 {
        sf.write(", ")?;
      }

      sf.write(value)?;
    }

    sf.write("]")
  }
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Object {
  pub properties: Vec<(Value, Value)>,
}

impl StructuredFormattable for Object {
  fn structured_fmt(&self, sf: &mut StructuredFormatter<'_, '_>) -> std::fmt::Result {
    if self.properties.is_empty() {
      return sf.write("{}");
    }

    sf.write("{ ")?;

    for (i, (key, value)) in self.properties.iter().enumerate() {
      if i > 0 {
        sf.write(", ")?;
      }

      sf.write_slice(&[key, &": ", value])?;
    }

    sf.write(" }")
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
