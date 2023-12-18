use std::collections::HashMap;
use std::str::FromStr;

use num_bigint::BigInt;
use valuescript_common::{InstructionByte, BUILTIN_NAMES};

use crate::asm::{
  Array, Builtin, Class, ContentHashable, Definition, DefinitionContent, ExportStar, FnLine,
  Function, Hash, Instruction, Label, LabelRef, Meta, Module, Number, Object, Pointer, Register,
  Value,
};

pub struct AssemblyParser<'a> {
  pub content: &'a str,
  pub pos: std::iter::Peekable<std::str::Chars<'a>>,
}

impl<'a> AssemblyParser<'a> {
  fn module(&mut self) -> Module {
    self.parse_exact("export");
    self.parse_whitespace();

    let export_default = self.assemble_value();
    self.parse_whitespace();

    let export_star = self.assemble_export_star();
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

    LineCol { line, col }
  }

  fn get_lines(&self) -> Vec<&str> {
    return self.content.split('\n').collect();
  }

  fn render_pos(&self, offset: isize, message: &str) -> String {
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

    true
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
      panic!("{}", self.render_pos(0, "Expected whitespace"));
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

  fn parse_line(&mut self) {
    loop {
      if let Some('\n') = self.pos.next() {
        return;
      }
    }
  }

  fn parse_optional_spaces(&mut self) {
    loop {
      match self.pos.peek() {
        Some(' ') => {}
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

    let content = 'b: {
      if self.test_chars("function") {
        break 'b DefinitionContent::Function(self.assemble_function());
      }

      if self.test_chars("meta") {
        break 'b DefinitionContent::Meta(self.assemble_fn_meta());
      }

      DefinitionContent::Value(self.assemble_value())
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
      ("const_apply", InstructionByte::Apply),
      ("bind", InstructionByte::Bind),
      ("sub", InstructionByte::Sub),
      ("submov", InstructionByte::SubMov),
      ("subcall", InstructionByte::SubCall),
      ("jmp", InstructionByte::Jmp),
      ("jmpif", InstructionByte::JmpIf),
      ("jmpif_not", InstructionByte::JmpIfNot),
      ("unary+", InstructionByte::UnaryPlus),
      ("unary-", InstructionByte::UnaryMinus),
      ("new", InstructionByte::New),
      ("throw", InstructionByte::Throw),
      ("import", InstructionByte::Import),
      ("import*", InstructionByte::ImportStar),
      ("set_catch", InstructionByte::SetCatch),
      ("unset_catch", InstructionByte::UnsetCatch),
      ("const_subcall", InstructionByte::ConstSubCall),
      ("require_mutable_this", InstructionByte::RequireMutableThis),
      ("this_subcall", InstructionByte::ThisSubCall),
      ("next", InstructionByte::Next),
      ("unpack_iter_res", InstructionByte::UnpackIterRes),
      ("cat", InstructionByte::Cat),
      ("yield", InstructionByte::Yield),
      ("yield*", InstructionByte::YieldStar),
      ("delete", InstructionByte::Delete),
    ]);

    for (word, instruction) in instruction_word_map {
      if self.test_instruction_word(word) {
        advance_chars(&mut self.pos, word.len());
        match self.pos.peek() {
          Some('\n') | None | Some(' ') => {}
          _ => panic!("Unexpected non-whitespace character after instruction word"),
        }
        self.parse_optional_spaces();
        return instruction;
      }
    }

    panic!("{}", self.render_pos(0, "Failed to parse instruction"));
  }

  fn test_instruction_word(&self, word: &str) -> bool {
    let mut pos = self.pos.clone();
    let has_chars = self.test_chars(word);

    if !has_chars {
      return false;
    }

    advance_chars(&mut pos, word.len());

    matches!(pos.next(), None | Some(' ') | Some('\n'))
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

    Some(res)
  }

  fn parse_identifier(&mut self) -> String {
    let optional_identifier = self.test_identifier();

    if optional_identifier.is_none() {
      panic!("{}", self.render_pos(0, "Invalid identifier"));
    }

    let identifier = optional_identifier.unwrap();
    advance_chars(&mut self.pos, identifier.len());

    identifier
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
          panic!("{}", self.render_pos(-1, "Unimplemented escape sequence"));
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
        self.render_pos(0, "Unexpected end of input after escape character")
      );
    }

    result
  }

  fn assemble_function(&mut self) -> Function {
    let mut function = Function::default();

    self.parse_exact("function");

    if self.test_chars("*") {
      advance_chars(&mut self.pos, 1);
      function.is_generator = true;
    }

    self.parse_whitespace();

    if self.test_chars("(") {
      // Leave meta as void
      self.parse_exact("(");
    } else {
      function.meta = Some(self.assemble_pointer());
      self.parse_optional_whitespace();
      self.parse_exact("(");
    }

    loop {
      self.parse_optional_whitespace();
      let mut next = self.parse_one_of(&["%", ")"]);

      if next == ")" {
        break;
      }

      if next != "%" {
        panic!("Expected this to be impossible");
      }

      let take = self.parse_one_of(&["!", ""]) == "!";

      let param_name = self.parse_identifier();

      function.parameters.push(Register {
        take,
        name: param_name.clone(),
      });

      next = self.parse_one_of(&[",", ")"]);

      if next == ")" {
        break;
      }
    }

    self.parse_optional_whitespace();
    self.parse_exact("{");
    self.parse_line();

    loop {
      self.parse_optional_spaces();

      let c = *self
        .pos
        .peek()
        .expect("Expected instruction, label, or end of function");

      if c == '\n' {
        self.pos.next();
        function.body.push(FnLine::Empty);
        continue;
      }

      if c == '}' {
        self.pos.next();
        break;
      }

      if c == '/' {
        self.parse_exact("//");

        let mut msg = String::new();

        loop {
          match self.pos.next() {
            Some('\n') | None => break,
            Some(c) => msg.push(c),
          }
        }

        function.body.push(FnLine::Comment(msg.trim().to_string()));

        continue;
      }

      if c == '(' {
        self.parse_exact("(release");
        self.parse_whitespace();

        let reg = self.assemble_register();
        self.parse_optional_whitespace();
        self.parse_exact(")\n");

        function.body.push(FnLine::Release(reg));

        continue;
      }

      let optional_label = self.test_label();

      function.body.push(match optional_label {
        Some(label) => FnLine::Label(self.assemble_label(label)),
        None => FnLine::Instruction(self.assemble_instruction()),
      });
    }

    function
  }

  fn assemble_fn_meta(&mut self) -> Meta {
    self.parse_exact("meta {");
    self.parse_optional_whitespace();

    self.parse_exact("name: ");

    let name = self.parse_string_literal();

    self.parse_exact(",");
    self.parse_optional_whitespace();

    let content_hashable = 'b: {
      if self.test_chars("}") {
        break 'b ContentHashable::Empty;
      }

      if self.test_chars("srcHash:") {
        self.parse_exact("srcHash: ");

        let src_hash = self.assemble_hash();

        self.parse_exact(",");
        self.parse_optional_whitespace();

        self.parse_exact("deps: ");
        let deps = self.assemble_array().values;
        self.parse_exact(",");
        self.parse_optional_whitespace();

        break 'b ContentHashable::Src(src_hash, deps);
      }

      if self.test_chars("contentHash:") {
        self.parse_exact("contentHash: ");

        let content_hash = self.assemble_hash();

        self.parse_exact(",");
        self.parse_optional_whitespace();

        break 'b ContentHashable::Content(content_hash);
      }

      panic!("{}", self.render_pos(-1, "Expected ContentHashable"));
    };

    self.parse_exact("}");

    Meta {
      name,
      content_hashable,
    }
  }

  fn assemble_class(&mut self) -> Class {
    self.parse_exact("class {");
    self.parse_optional_whitespace();

    self.parse_exact("meta: ");
    let meta = self.assemble_fn_meta();
    self.parse_exact(",");
    self.parse_optional_whitespace();

    self.parse_exact("constructor: ");
    let constructor = self.assemble_value();
    self.parse_exact(",");
    self.parse_optional_whitespace();

    self.parse_exact("prototype: ");
    let prototype = self.assemble_value();
    self.parse_exact(",");
    self.parse_optional_whitespace();

    self.parse_exact("static: ");
    let static_ = self.assemble_value();
    self.parse_exact(",");
    self.parse_optional_whitespace();

    self.parse_exact("}");

    Class {
      meta,
      constructor,
      prototype,
      static_,
    }
  }

  fn assemble_instruction(&mut self) -> Instruction {
    use InstructionByte::*;

    let instr = self.parse_instruction_word();

    let res = match instr {
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
        self.assemble_register(),
        self.assemble_value(),
        self.assemble_register(),
      ),
      ConstApply => Instruction::ConstApply(
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
        self.assemble_register(),
        self.assemble_value(),
        self.assemble_value(),
        self.assemble_register(),
      ),
      Jmp => Instruction::Jmp(self.assemble_label_read()),
      JmpIf => Instruction::JmpIf(self.assemble_value(), self.assemble_label_read()),
      JmpIfNot => Instruction::JmpIfNot(self.assemble_value(), self.assemble_label_read()),
      UnaryPlus => Instruction::UnaryPlus(self.assemble_value(), self.assemble_register()),
      UnaryMinus => Instruction::UnaryMinus(self.assemble_value(), self.assemble_register()),
      New => Instruction::New(
        self.assemble_value(),
        self.assemble_value(),
        self.assemble_register(),
      ),
      Throw => Instruction::Throw(self.assemble_value()),
      Import => Instruction::Import(self.assemble_value(), self.assemble_register()),
      ImportStar => Instruction::ImportStar(self.assemble_value(), self.assemble_register()),
      SetCatch => Instruction::SetCatch(self.assemble_label_read(), self.assemble_register()),
      UnsetCatch => Instruction::UnsetCatch,
      ConstSubCall => Instruction::ConstSubCall(
        self.assemble_value(),
        self.assemble_value(),
        self.assemble_value(),
        self.assemble_register(),
      ),
      RequireMutableThis => Instruction::RequireMutableThis,
      ThisSubCall => Instruction::ThisSubCall(
        self.assemble_register(),
        self.assemble_value(),
        self.assemble_value(),
        self.assemble_register(),
      ),
      Next => Instruction::Next(self.assemble_register(), self.assemble_register()),
      UnpackIterRes => Instruction::UnpackIterRes(
        self.assemble_register(),
        self.assemble_register(),
        self.assemble_register(),
      ),
      Cat => Instruction::Cat(self.assemble_value(), self.assemble_register()),
      Yield => Instruction::Yield(self.assemble_value(), self.assemble_register()),
      YieldStar => Instruction::YieldStar(self.assemble_value(), self.assemble_register()),
      Delete => Instruction::Delete(
        self.assemble_register(),
        self.assemble_value(),
        self.assemble_register(),
      ),
    };

    self.parse_line();

    res
  }

  pub fn assemble_value(&mut self) -> Value {
    self.parse_optional_whitespace();

    match self.pos.peek() {
      None => {
        panic!("{}", self.render_pos(0, "Expected value"));
      }
      Some('%') => Value::Register(self.assemble_register()),
      Some('@') => Value::Pointer(self.assemble_pointer()),
      Some('$') => Value::Builtin(self.assemble_builtin()),
      Some('[') => Value::Array(Box::new(self.assemble_array())),
      Some('-' | '.' | '0'..='9') => self.assemble_number(),
      Some('"') => Value::String(self.parse_string_literal()),
      Some('{') => Value::Object(Box::new(self.assemble_object())),
      Some('c') => Value::Class(Box::new(self.assemble_class())),
      Some(ref_c) => {
        let c = *ref_c;

        let parsed = self.parse_one_of(&[
          "void",
          "undefined",
          "null",
          "false",
          "true",
          "Infinity",
          "NaN",
          "",
        ]);

        match parsed.as_str() {
          "void" => Value::Void,
          "undefined" => Value::Undefined,
          "null" => Value::Null,
          "false" => Value::Bool(false),
          "true" => Value::Bool(true),
          "Infinity" => Value::Number(Number(f64::INFINITY)),
          "NaN" => Value::Number(Number(f64::NAN)),

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

  fn assemble_pointer(&mut self) -> Pointer {
    self.parse_exact("@");
    let name = self.parse_identifier();
    Pointer { name }
  }

  fn assemble_array(&mut self) -> Array {
    let mut array = Array::default();

    self.parse_optional_whitespace();

    self.parse_exact("[");

    loop {
      self.parse_optional_whitespace();

      match self.pos.peek() {
        None => {
          panic!("{}", self.render_pos(0, "Expected value or array end"));
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
        self.parse_optional_whitespace();
        continue;
      }

      if next == "]" {
        break array;
      }

      panic!("Expected this to be impossible");
    }
  }

  fn assemble_register(&mut self) -> Register {
    self.parse_optional_whitespace();
    self.parse_exact("%");
    let take = self.parse_one_of(&["!", ""]) == "!";
    let name = self.parse_identifier();

    Register { take, name }
  }

  fn assemble_builtin(&mut self) -> Builtin {
    self.parse_exact("$");

    let name = self.parse_identifier();

    if !BUILTIN_NAMES.contains(&name.as_str()) {
      panic!("Unrecognized builtin ${}", name);
    }

    Builtin { name }
  }

  fn test_label(&self) -> Option<String> {
    let identifier = self.test_identifier()?;

    let mut pos = self.pos.clone();
    advance_chars(&mut pos, identifier.len());

    if pos.next() == Some(':') {
      return Some(identifier);
    }

    None
  }

  fn assemble_label(&mut self, name: String) -> Label {
    self.parse_line();
    Label { name }
  }

  fn assemble_label_read(&mut self) -> LabelRef {
    self.parse_optional_whitespace();
    self.parse_exact(":");
    let name = self.parse_identifier();

    LabelRef { name }
  }

  fn assemble_number(&mut self) -> Value {
    if self.parse_one_of(&["-Infinity", ""]) == "-Infinity" {
      return Value::Number(Number(f64::NEG_INFINITY));
    }

    let mut num_string = "".to_string();

    while let Some('-' | '.' | 'e' | 'n' | '0'..='9') = self.pos.peek() {
      num_string.push(self.pos.next().unwrap());
    }

    if num_string.ends_with('n') {
      num_string.pop();

      match BigInt::parse_bytes(num_string.as_bytes(), 10) {
        Some(bigint) => return Value::BigInt(bigint),
        None => {
          panic!(
            "{}",
            self.render_pos(-(num_string.len() as isize + 1), "Expected valid number")
          );
        }
      }
    }

    let value_result = f64::from_str(num_string.as_str());

    if value_result.is_err() {
      panic!(
        "{}",
        self.render_pos(-(num_string.len() as isize), "Expected valid number")
      );
    }

    Value::Number(Number(value_result.unwrap()))
  }

  fn assemble_object(&mut self) -> Object {
    let mut object = Object::default();

    self.parse_exact("{");

    loop {
      match self.assemble_object_kv() {
        None => break object,
        Some(kv) => object.properties.push(kv),
      };

      self.parse_optional_whitespace();

      let c = *self.pos.peek().expect("Expected comma or object end");

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

  fn assemble_object_kv(&mut self) -> Option<(Value, Value)> {
    self.parse_optional_whitespace();
    let c = *self.pos.peek().expect("Expected object content or end");

    let key = match c {
      '}' => {
        self.pos.next();
        return None;
      }
      _ => self.assemble_value(),
    };

    self.parse_optional_whitespace();
    self.parse_exact(":");
    let value = self.assemble_value();

    Some((key, value))
  }

  fn assemble_export_star(&mut self) -> ExportStar {
    let mut export_star = ExportStar::default();

    self.parse_exact("{");

    loop {
      self.parse_optional_whitespace();

      if self.parse_one_of(&["include ", ""]) == "" {
        break;
      }

      export_star.includes.push(self.assemble_pointer());
      self.parse_optional_whitespace();

      let c = *self.pos.peek().expect("Expected comma or object end");

      match c {
        ',' => {
          self.pos.next();
        }
        '}' => {
          self.pos.next();
          return export_star;
        }
        _ => {
          panic!(
            "{}",
            self.render_pos(0, &format!("Unexpected character {}", c))
          );
        }
      }
    }

    loop {
      match self.assemble_object_kv() {
        None => break export_star,
        Some(kv) => export_star.local.properties.push(kv),
      };

      self.parse_optional_whitespace();

      let c = *self.pos.peek().expect("Expected comma or object end");

      match c {
        ',' => {
          self.pos.next();
        }
        '}' => {
          self.pos.next();
          break export_star;
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

  fn assemble_hash(&mut self) -> Hash {
    self.parse_exact("#");

    let mut res = [0u8; 32];

    for res_byte in &mut res {
      *res_byte = match self.assemble_hex_byte() {
        Some(b) => b,
        None => panic!("{}", self.render_pos(0, "Expected hex byte")),
      }
    }

    Hash(res)
  }

  fn assemble_hex_byte(&mut self) -> Option<u8> {
    let high_nibble = self.pos.next()?.to_digit(16)? as u8;
    let low_nibble = self.pos.next()?.to_digit(16)? as u8;

    Some((high_nibble << 4) | low_nibble)
  }
}

pub fn parse_module(content: &str) -> Module {
  let mut assembler = AssemblyParser {
    content,
    pos: content.chars().peekable(),
  };

  assembler.module()
}

fn is_leading_identifier_char(c: char) -> bool {
  c == '_' || c.is_ascii_alphabetic()
}

fn is_identifier_char(c: char) -> bool {
  c == '_' || c.is_ascii_alphanumeric()
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
