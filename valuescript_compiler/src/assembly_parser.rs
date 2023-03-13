use std::collections::HashMap;
use std::str::FromStr;

use crate::asm::{
  Array, Builtin, Class, Definition, DefinitionContent, Function, Instruction, InstructionOrLabel,
  Label, LabelRef, Module, Object, Pointer, Register, Value,
};

struct AssemblyParser<'a> {
  content: &'a str,
  pos: std::iter::Peekable<std::str::Chars<'a>>,
}

impl<'a> AssemblyParser<'a> {
  fn module(&mut self) -> Module {
    self.parse_exact("export");
    self.parse_whitespace();

    let export_default = self.assemble_value();
    self.parse_whitespace();

    let export_star = self.assemble_object();
    self.parse_whitespace();

    let mut definitions = Vec::<Definition>::new();

    loop {
      self.parse_optional_whitespace();

      if self.pos.peek().is_none() {
        break;
      }

      definitions.push(self.assemble_definition());
    }

    Module {
      export_default,
      export_star,
      definitions,
    }
  }

  fn get_pos_index(&self) -> usize {
    let mut start = self.content.chars();
    let mut i = 0_usize;

    loop {
      if start.clone().eq(self.pos.clone()) {
        return i;
      }

      i += 1;
      start.next();
    }
  }

  fn get_line_col(&self, offset: isize) -> LineCol {
    let mut line = 1;
    let mut col = 1;

    let raw_pos = self.get_pos_index();
    let pos = (raw_pos as isize + offset).max(0) as usize;

    for (i, c) in self.content.chars().enumerate() {
      if i == pos {
        break;
      }

      if c == '\n' {
        line += 1;
        col = 1;
      } else {
        col += 1;
      }
    }

    return LineCol { line, col };
  }

  fn get_lines(&self) -> Vec<&str> {
    return self.content.split('\n').collect();
  }

  fn render_pos(&self, offset: isize, message: &String) -> String {
    let LineCol { line, col } = self.get_line_col(offset);
    let source_lines = self.get_lines();

    let mut output = String::new();

    output += &format!("(unknown):{}:{}: {}:\n", line, col, message);

    if line >= 2 {
      output += &format!("{: >6} | {}\n", line - 1, source_lines[line - 2]);
    }

    output += &format!("{: >6} | {}\n", line, source_lines[line - 1]);
    output += &format!("{: >6} | {}\n", "", " ".repeat(col - 1) + "^");

    if line < source_lines.len() {
      output += &format!("{: >6} | {}\n", line + 1, source_lines[line]);
    }

    output
  }

  fn test_chars(&self, chars: &str) -> bool {
    let mut pos = self.pos.clone();

    for c in chars.chars() {
      if pos.next() != Some(c) {
        return false;
      }
    }

    return true;
  }

  fn parse_whitespace(&mut self) {
    let mut count = 0;

    loop {
      match self.pos.peek() {
        Some(' ') => (),
        Some('\n') => (),
        _ => break,
      }

      count += 1;
      self.pos.next();
    }

    if count == 0 {
      panic!("{}", self.render_pos(0, &"Expected whitespace".to_string()));
    }
  }

  fn parse_optional_whitespace(&mut self) {
    loop {
      match self.pos.peek() {
        Some(' ') => {}
        Some('\n') => {}
        _ => {
          return;
        }
      }

      self.pos.next();
    }
  }

  fn assemble_definition(&mut self) -> Definition {
    self.parse_exact("@");
    let def_name = self.parse_identifier();
    self.parse_optional_whitespace();
    self.parse_exact("=");
    self.parse_optional_whitespace();

    let c = *self.pos.peek().expect("Expected value for definition");

    let content = match c {
      'f' => DefinitionContent::Function(self.assemble_function()),
      'c' => DefinitionContent::Class(self.assemble_class()),
      _ => DefinitionContent::Value(self.assemble_value()),
    };

    Definition {
      pointer: Pointer { name: def_name },
      content,
    }
  }

  fn parse_instruction_word(&mut self) -> InstructionByte {
    let instruction_word_map: HashMap<&str, InstructionByte> = HashMap::from([
      ("end", InstructionByte::End),
      ("mov", InstructionByte::Mov),
      ("op++", InstructionByte::OpInc),
      ("op--", InstructionByte::OpDec),
      ("op+", InstructionByte::OpPlus),
      ("op-", InstructionByte::OpMinus),
      ("op*", InstructionByte::OpMul),
      ("op/", InstructionByte::OpDiv),
      ("op%", InstructionByte::OpMod),
      ("op**", InstructionByte::OpExp),
      ("op==", InstructionByte::OpEq),
      ("op!=", InstructionByte::OpNe),
      ("op===", InstructionByte::OpTripleEq),
      ("op!==", InstructionByte::OpTripleNe),
      ("op&&", InstructionByte::OpAnd),
      ("op||", InstructionByte::OpOr),
      ("op!", InstructionByte::OpNot),
      ("op<", InstructionByte::OpLess),
      ("op<=", InstructionByte::OpLessEq),
      ("op>", InstructionByte::OpGreater),
      ("op>=", InstructionByte::OpGreaterEq),
      ("op??", InstructionByte::OpNullishCoalesce),
      ("op?.", InstructionByte::OpOptionalChain),
      ("op&", InstructionByte::OpBitAnd),
      ("op|", InstructionByte::OpBitOr),
      ("op~", InstructionByte::OpBitNot),
      ("op^", InstructionByte::OpBitXor),
      ("op<<", InstructionByte::OpLeftShift),
      ("op>>", InstructionByte::OpRightShift),
      ("op>>>", InstructionByte::OpRightShiftUnsigned),
      ("typeof", InstructionByte::TypeOf),
      ("instanceof", InstructionByte::InstanceOf),
      ("in", InstructionByte::In),
      ("call", InstructionByte::Call),
      ("apply", InstructionByte::Apply),
      ("bind", InstructionByte::Bind),
      ("sub", InstructionByte::Sub),
      ("submov", InstructionByte::SubMov),
      ("subcall", InstructionByte::SubCall),
      ("jmp", InstructionByte::Jmp),
      ("jmpif", InstructionByte::JmpIf),
      ("unary+", InstructionByte::UnaryPlus),
      ("unary-", InstructionByte::UnaryMinus),
      ("new", InstructionByte::New),
    ]);

    for (word, instruction) in instruction_word_map {
      if self.test_instruction_word(word) {
        advance_chars(&mut self.pos, word.len() + 1);
        self.parse_optional_whitespace();
        return instruction;
      }
    }

    panic!(
      "{}",
      self.render_pos(0, &"Failed to parse instruction".to_string())
    );
  }

  fn test_instruction_word(&self, word: &str) -> bool {
    let mut pos = self.pos.clone();
    let has_chars = self.test_chars(word);

    if !has_chars {
      return false;
    }

    advance_chars(&mut pos, word.len());

    return match pos.next() {
      None => true,
      Some(' ') => true,
      Some('\n') => true,
      _ => false,
    };
  }

  fn test_identifier(&self) -> Option<String> {
    let start = self.pos.clone();
    let mut pos = start;
    let mut res = "".to_string();

    let leading_char = match pos.next() {
      None => {
        return None;
      }
      Some(c) => c,
    };

    if !is_leading_identifier_char(leading_char) {
      return None;
    }

    res.push(leading_char);

    loop {
      match pos.next() {
        None => {
          break;
        }
        Some(c) => {
          if !is_identifier_char(c) {
            break;
          }

          res.push(c);
        }
      };
    }

    return Some(res);
  }

  fn parse_identifier(&mut self) -> String {
    let optional_identifier = self.test_identifier();

    if optional_identifier.is_none() {
      panic!("{}", self.render_pos(0, &"Invalid identifier".to_string()));
    }

    let identifier = optional_identifier.unwrap();
    advance_chars(&mut self.pos, identifier.len());

    return identifier;
  }

  fn parse_exact(&mut self, chars: &str) {
    for c in chars.chars() {
      if self.pos.next() != Some(c) {
        panic!("{}", self.render_pos(-1, &format!("Expected '{}'", c)));
      }
    }
  }

  fn parse_one_of(&mut self, options: &[&str]) -> String {
    for opt in options {
      if self.test_chars(opt) {
        advance_chars(&mut self.pos, opt.len());
        return opt.to_string();
      }
    }

    panic!(
      "{}",
      self.render_pos(0, &format!("Expected one of {:?}", options))
    );
  }

  fn parse_string_literal(&mut self) -> String {
    let mut result = "".to_string();

    self.parse_exact("\"");
    let mut escaping = false;

    loop {
      let oc = self.pos.next();

      if oc.is_none() {
        break;
      }

      let c = oc.unwrap();

      if escaping {
        if c == '\\' {
          result.push('\\');
        } else if c == '"' {
          result.push('"');
        } else if c == 'n' {
          result.push('\n');
        } else if c == 't' {
          result.push('\t');
        } else {
          panic!(
            "{}",
            self.render_pos(-1, &"Unimplemented escape sequence".to_string())
          );
        }

        escaping = false;
      } else if c == '\\' {
        escaping = true;
      } else if c == '"' {
        break;
      } else {
        result.push(c);
      }
    }

    if escaping {
      panic!(
        "{}",
        self.render_pos(
          0,
          &"Unexpected end of input after escape character".to_string()
        )
      );
    }

    return result;
  }

  fn assemble_function(&mut self) -> Function {
    let mut function = Function::default();

    self.parse_exact("function(");

    loop {
      self.parse_optional_whitespace();
      let mut next = self.parse_one_of(&["%", ")"]);

      if next == ")" {
        break;
      }

      if next != "%" {
        panic!("Expected this to be impossible");
      }

      let param_name = self.parse_identifier();

      function
        .parameters
        .push(Register::Named(param_name.clone()));

      next = self.parse_one_of(&[",", ")"]);

      if next == ")" {
        break;
      }
    }

    self.parse_optional_whitespace();
    self.parse_exact("{");

    loop {
      self.parse_optional_whitespace();

      let c = *self
        .pos
        .peek()
        .expect("Expected instruction, label, or end of function");

      if c == '}' {
        self.pos.next();
        break;
      }

      let optional_label = self.test_label();

      if optional_label.is_some() {
        function.body.push(InstructionOrLabel::Label(
          self.assemble_label(optional_label.unwrap()),
        ));
      } else {
        function
          .body
          .push(InstructionOrLabel::Instruction(self.assemble_instruction()));
      }
    }

    function
  }

  fn assemble_class(&mut self) -> Class {
    self.parse_exact("class(");
    self.parse_optional_whitespace();

    let constructor = self.assemble_value();
    self.parse_optional_whitespace();

    self.parse_exact(",");
    self.parse_optional_whitespace();

    let methods = self.assemble_value();
    self.parse_optional_whitespace();

    self.parse_exact(")");

    Class {
      constructor,
      methods,
    }
  }

  fn assemble_instruction(&mut self) -> Instruction {
    use InstructionByte::*;

    let instr = self.parse_instruction_word();

    match instr {
      End => Instruction::End,
      Mov => Instruction::Mov(self.assemble_value(), self.assemble_register()),
      OpInc => Instruction::OpInc(self.assemble_register()),
      OpDec => Instruction::OpDec(self.assemble_register()),
      OpPlus => Instruction::OpPlus(
        self.assemble_value(),
        self.assemble_value(),
        self.assemble_register(),
      ),
      OpMinus => Instruction::OpMinus(
        self.assemble_value(),
        self.assemble_value(),
        self.assemble_register(),
      ),
      OpMul => Instruction::OpMul(
        self.assemble_value(),
        self.assemble_value(),
        self.assemble_register(),
      ),
      OpDiv => Instruction::OpDiv(
        self.assemble_value(),
        self.assemble_value(),
        self.assemble_register(),
      ),
      OpMod => Instruction::OpMod(
        self.assemble_value(),
        self.assemble_value(),
        self.assemble_register(),
      ),
      OpExp => Instruction::OpExp(
        self.assemble_value(),
        self.assemble_value(),
        self.assemble_register(),
      ),
      OpEq => Instruction::OpEq(
        self.assemble_value(),
        self.assemble_value(),
        self.assemble_register(),
      ),
      OpNe => Instruction::OpNe(
        self.assemble_value(),
        self.assemble_value(),
        self.assemble_register(),
      ),
      OpTripleEq => Instruction::OpTripleEq(
        self.assemble_value(),
        self.assemble_value(),
        self.assemble_register(),
      ),
      OpTripleNe => Instruction::OpTripleNe(
        self.assemble_value(),
        self.assemble_value(),
        self.assemble_register(),
      ),
      OpAnd => Instruction::OpAnd(
        self.assemble_value(),
        self.assemble_value(),
        self.assemble_register(),
      ),
      OpOr => Instruction::OpOr(
        self.assemble_value(),
        self.assemble_value(),
        self.assemble_register(),
      ),
      OpNot => Instruction::OpNot(self.assemble_value(), self.assemble_register()),
      OpLess => Instruction::OpLess(
        self.assemble_value(),
        self.assemble_value(),
        self.assemble_register(),
      ),
      OpLessEq => Instruction::OpLessEq(
        self.assemble_value(),
        self.assemble_value(),
        self.assemble_register(),
      ),
      OpGreater => Instruction::OpGreater(
        self.assemble_value(),
        self.assemble_value(),
        self.assemble_register(),
      ),
      OpGreaterEq => Instruction::OpGreaterEq(
        self.assemble_value(),
        self.assemble_value(),
        self.assemble_register(),
      ),
      OpNullishCoalesce => Instruction::OpNullishCoalesce(
        self.assemble_value(),
        self.assemble_value(),
        self.assemble_register(),
      ),
      OpOptionalChain => Instruction::OpOptionalChain(
        self.assemble_value(),
        self.assemble_value(),
        self.assemble_register(),
      ),
      OpBitAnd => Instruction::OpBitAnd(
        self.assemble_value(),
        self.assemble_value(),
        self.assemble_register(),
      ),
      OpBitOr => Instruction::OpBitOr(
        self.assemble_value(),
        self.assemble_value(),
        self.assemble_register(),
      ),
      OpBitNot => Instruction::OpBitNot(self.assemble_value(), self.assemble_register()),
      OpBitXor => Instruction::OpBitXor(
        self.assemble_value(),
        self.assemble_value(),
        self.assemble_register(),
      ),
      OpLeftShift => Instruction::OpLeftShift(
        self.assemble_value(),
        self.assemble_value(),
        self.assemble_register(),
      ),
      OpRightShift => Instruction::OpRightShift(
        self.assemble_value(),
        self.assemble_value(),
        self.assemble_register(),
      ),
      OpRightShiftUnsigned => Instruction::OpRightShiftUnsigned(
        self.assemble_value(),
        self.assemble_value(),
        self.assemble_register(),
      ),
      TypeOf => Instruction::TypeOf(self.assemble_value(), self.assemble_register()),
      InstanceOf => Instruction::InstanceOf(
        self.assemble_value(),
        self.assemble_value(),
        self.assemble_register(),
      ),
      In => Instruction::In(
        self.assemble_value(),
        self.assemble_value(),
        self.assemble_register(),
      ),
      Call => Instruction::Call(
        self.assemble_value(),
        self.assemble_value(),
        self.assemble_register(),
      ),
      Apply => Instruction::Apply(
        self.assemble_value(),
        self.assemble_value(),
        self.assemble_value(),
        self.assemble_register(),
      ),
      Bind => Instruction::Bind(
        self.assemble_value(),
        self.assemble_value(),
        self.assemble_register(),
      ),
      Sub => Instruction::Sub(
        self.assemble_value(),
        self.assemble_value(),
        self.assemble_register(),
      ),
      SubMov => Instruction::SubMov(
        self.assemble_value(),
        self.assemble_value(),
        self.assemble_register(),
      ),
      SubCall => Instruction::SubCall(
        self.assemble_value(),
        self.assemble_value(),
        self.assemble_value(),
        self.assemble_register(),
      ),
      Jmp => Instruction::Jmp(self.assemble_label_read()),
      JmpIf => Instruction::JmpIf(self.assemble_value(), self.assemble_label_read()),
      UnaryPlus => Instruction::UnaryPlus(self.assemble_value(), self.assemble_register()),
      UnaryMinus => Instruction::UnaryMinus(self.assemble_value(), self.assemble_register()),
      New => Instruction::New(
        self.assemble_value(),
        self.assemble_value(),
        self.assemble_register(),
      ),
    }
  }

  fn assemble_value(&mut self) -> Value {
    self.parse_optional_whitespace();

    match self.pos.peek() {
      None => {
        panic!("{}", self.render_pos(0, &format!("Expected value")));
      }
      Some('%') => Value::Register(self.assemble_register()),
      Some('@') => {
        self.parse_exact("@");
        let name = self.parse_identifier();
        Value::Pointer(Pointer { name })
      }
      Some('$') => Value::Builtin(self.assemble_builtin()),
      Some('[') => Value::Array(Box::new(self.assemble_array())),
      Some('-' | '.' | '0'..='9') => Value::Number(self.assemble_number()),
      Some('"') => Value::String(self.parse_string_literal()),
      Some('{') => Value::Object(Box::new(self.assemble_object())),
      Some(ref_c) => {
        let c = *ref_c;

        let parsed = self.parse_one_of(&["void", "undefined", "null", "false", "true", ""]);

        match parsed.as_str() {
          "void" => Value::Void,
          "undefined" => Value::Undefined,
          "null" => Value::Null,
          "false" => Value::Bool(false),
          "true" => Value::Bool(true),

          // TODO: Finish implementing the different values
          _ => {
            panic!(
              "{}",
              self.render_pos(
                -(parsed.len() as isize),
                &format!("Unimplemented value type or unexpected character {}", c)
              )
            );
          }
        }
      }
    }
  }

  fn assemble_array(&mut self) -> Array {
    let mut array = Array::default();

    self.parse_optional_whitespace();

    self.parse_exact("[");

    loop {
      self.parse_optional_whitespace();

      match self.pos.peek() {
        None => {
          panic!(
            "{}",
            self.render_pos(0, &format!("Expected value or array end"))
          );
        }
        Some(']') => {
          self.pos.next();
          break array;
        }
        _ => {}
      }

      array.values.push(self.assemble_value());
      self.parse_optional_whitespace();

      let next = self.parse_one_of(&[",", "]"]);

      if next == "," {
        self.pos.next(); // TODO: Assert whitespace
        continue;
      }

      if next == "]" {
        self.parse_optional_whitespace();
        break array;
      }

      panic!("Expected this to be impossible");
    }
  }

  fn assemble_register(&mut self) -> Register {
    self.parse_optional_whitespace();
    self.parse_exact("%");
    let name = self.parse_identifier();

    match name.as_str() {
      "return" => Register::Return,
      "this" => Register::This,
      "ignore" => Register::Ignore,
      _ => Register::Named(name),
    }
  }

  fn assemble_builtin(&mut self) -> Builtin {
    match self.parse_one_of(&["$Math", "$Debug", "$String"]).as_str() {
      "$Math" => Builtin {
        name: "Math".to_string(),
      },
      "$Debug" => Builtin {
        name: "Debug".to_string(),
      },
      "$String" => Builtin {
        name: "String".to_string(),
      },
      _ => panic!("Shouldn't happen"),
    }
  }

  fn test_label(&self) -> Option<String> {
    let optional_identifier = self.test_identifier();

    if optional_identifier.is_none() {
      return None;
    }

    let identifier = optional_identifier.unwrap();

    let mut pos = self.pos.clone();
    advance_chars(&mut pos, identifier.len());

    if pos.next() == Some(':') {
      return Some(identifier);
    }

    return None;
  }

  fn assemble_label(&mut self, name: String) -> Label {
    self.parse_optional_whitespace();
    advance_chars(&mut self.pos, name.len() + 1);

    Label { name }
  }

  fn assemble_label_read(&mut self) -> LabelRef {
    self.parse_optional_whitespace();
    self.parse_exact(":");
    let name = self.parse_identifier();

    LabelRef { name }
  }

  fn assemble_number(&mut self) -> f64 {
    let mut num_string = "".to_string();

    loop {
      match self.pos.peek() {
        Some('-' | '.' | 'e' | '0'..='9') => {
          num_string.push(self.pos.next().unwrap());
        }
        _ => {
          break;
        }
      }
    }

    let value_result = f64::from_str(num_string.as_str());

    if value_result.is_err() {
      panic!(
        "{}",
        self.render_pos(
          -(num_string.len() as isize),
          &format!("Expected valid number")
        )
      );
    }

    value_result.unwrap()
  }

  fn assemble_object(&mut self) -> Object {
    let mut object = Object::default();

    self.parse_exact("{");

    loop {
      self.parse_optional_whitespace();
      let mut c = *self.pos.peek().expect("Expected object content or end");

      let key = match c {
        '"' => Value::String(self.parse_string_literal()),
        '%' => Value::Register(self.assemble_register()),
        '@' => {
          self.parse_exact("@");
          let name = self.parse_identifier();
          Value::Pointer(Pointer { name })
        }
        '}' => {
          self.pos.next();
          break object;
        }
        _ => {
          panic!(
            "{}",
            self.render_pos(0, &format!("Unexpected character {}", c))
          );
        }
      };

      self.parse_optional_whitespace();
      self.parse_exact(":");
      let value = self.assemble_value();

      object.properties.push((key, value));

      self.parse_optional_whitespace();

      c = *self.pos.peek().expect("Expected comma or object end");

      match c {
        ',' => {
          self.pos.next();
        }
        '}' => {
          self.pos.next();
          break object;
        }
        _ => {
          panic!(
            "{}",
            self.render_pos(0, &format!("Unexpected character {}", c))
          );
        }
      }
    }
  }
}

pub fn parse_module(content: &str) -> Module {
  let mut assembler = AssemblyParser {
    content,
    pos: content.chars().peekable(),
  };

  assembler.module()
}

#[derive(Debug, Clone)]
enum InstructionByte {
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
}

fn is_leading_identifier_char(c: char) -> bool {
  return c == '_' || ('a' <= c && c <= 'z') || ('A' <= c && c <= 'Z');
}

fn is_identifier_char(c: char) -> bool {
  return c == '_' || ('0' <= c && c <= '9') || ('a' <= c && c <= 'z') || ('A' <= c && c <= 'Z');
}

fn advance_chars(iter: &mut std::iter::Peekable<std::str::Chars>, len: usize) {
  for _ in 0..len {
    iter.next();
  }
}

struct LineCol {
  line: usize,
  col: usize,
}

impl std::fmt::Display for LineCol {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "line {} col {}", self.line, self.col)
  }
}
