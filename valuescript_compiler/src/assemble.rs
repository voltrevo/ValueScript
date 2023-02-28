use std::collections::HashMap;
use std::rc::Rc;
use std::str::FromStr;

#[derive(Default)]
struct LocationMap {
  references: HashMap<String, Vec<usize>>,
  found_locations: HashMap<String, usize>,
}

trait LocationMapper {
  fn add_unresolved(&mut self, name: &String, output: &mut Vec<u8>);
  fn resolve(&self, output: &mut Vec<u8>);
}

impl LocationMapper for LocationMap {
  fn add_unresolved(&mut self, name: &String, output: &mut Vec<u8>) {
    self
      .references
      .entry(name.clone())
      .or_default()
      .push(output.len());

    output.push(0xff);
    output.push(0xff); // TODO: Support >65535
  }

  fn resolve(&self, output: &mut Vec<u8>) {
    for (name, ref_locations) in &self.references {
      let location_optional = self.found_locations.get(name);

      if location_optional.is_none() {
        std::panic!("Unresolved reference to {} at {}", name, ref_locations[0]);
      }

      let location = location_optional.unwrap();

      for ref_location in ref_locations {
        output[*ref_location] = (*location % 256) as u8;
        output[*ref_location + 1] = (*location / 256) as u8; // TODO: Support >65535
      }
    }
  }
}

#[derive(Default)]
struct AssemblerFnData {
  register_map: HashMap<String, u8>,
  register_count_pos: usize,
  labels_map: LocationMap,
}

struct Assembler<'a> {
  content: &'a str,
  pos: std::iter::Peekable<std::str::Chars<'a>>,
  output: Vec<u8>,
  fn_data: AssemblerFnData,
  definitions_map: LocationMap,
}

impl<'a> Assembler<'a> {
  fn run(&mut self) {
    loop {
      self.parse_optional_whitespace();

      if self.pos.peek().is_none() {
        break;
      }

      self.assemble_definition();
    }

    self.definitions_map.resolve(&mut self.output);
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

  fn test_chars(&self, chars: &str) -> bool {
    let mut pos = self.pos.clone();

    for c in chars.chars() {
      if pos.next() != Some(c) {
        return false;
      }
    }

    return true;
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

  fn assemble_definition(&mut self) {
    self.parse_exact("@");
    let def_name = self.parse_identifier();
    self
      .definitions_map
      .found_locations
      .insert(def_name, self.output.len());
    self.parse_optional_whitespace();
    self.parse_exact("=");
    self.parse_optional_whitespace();

    let c = *self.pos.peek().expect("Expected value for definition");

    if c == 'f' {
      self.assemble_function();
    } else if c == 'c' {
      self.assemble_class();
    } else {
      self.assemble_value();
    }
  }

  fn parse_instruction_word(&mut self) -> Instruction {
    let instruction_word_map: HashMap<&str, Instruction> = HashMap::from([
      ("end", Instruction::End),
      ("mov", Instruction::Mov),
      ("op++", Instruction::OpInc),
      ("op--", Instruction::OpDec),
      ("op+", Instruction::OpPlus),
      ("op-", Instruction::OpMinus),
      ("op*", Instruction::OpMul),
      ("op/", Instruction::OpDiv),
      ("op%", Instruction::OpMod),
      ("op**", Instruction::OpExp),
      ("op==", Instruction::OpEq),
      ("op!=", Instruction::OpNe),
      ("op===", Instruction::OpTripleEq),
      ("op!==", Instruction::OpTripleNe),
      ("op&&", Instruction::OpAnd),
      ("op||", Instruction::OpOr),
      ("op!", Instruction::OpNot),
      ("op<", Instruction::OpLess),
      ("op<=", Instruction::OpLessEq),
      ("op>", Instruction::OpGreater),
      ("op>=", Instruction::OpGreaterEq),
      ("op??", Instruction::OpNullishCoalesce),
      ("op?.", Instruction::OpOptionalChain),
      ("op&", Instruction::OpBitAnd),
      ("op|", Instruction::OpBitOr),
      ("op~", Instruction::OpBitNot),
      ("op^", Instruction::OpBitXor),
      ("op<<", Instruction::OpLeftShift),
      ("op>>", Instruction::OpRightShift),
      ("op>>>", Instruction::OpRightShiftUnsigned),
      ("typeof", Instruction::TypeOf),
      ("instanceof", Instruction::InstanceOf),
      ("in", Instruction::In),
      ("call", Instruction::Call),
      ("apply", Instruction::Apply),
      ("bind", Instruction::Bind),
      ("sub", Instruction::Sub),
      ("submov", Instruction::SubMov),
      ("subcall", Instruction::SubCall),
      ("jmp", Instruction::Jmp),
      ("jmpif", Instruction::JmpIf),
      ("unary+", Instruction::UnaryPlus),
      ("unary-", Instruction::UnaryMinus),
      ("new", Instruction::New),
    ]);

    for (word, instruction) in instruction_word_map {
      if self.test_instruction_word(word) {
        advance_chars(&mut self.pos, word.len() + 1);
        self.parse_optional_whitespace();
        return instruction;
      }
    }

    std::panic!("Failed to parse instruction at {}", self.get_pos_index());
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
      std::panic!("Invalid identifier at {}", self.get_pos_index());
    }

    let identifier = optional_identifier.unwrap();
    advance_chars(&mut self.pos, identifier.len());

    return identifier;
  }

  fn parse_exact(&mut self, chars: &str) {
    for c in chars.chars() {
      if self.pos.next() != Some(c) {
        std::panic!("Expected '{}' at {}", c, self.get_pos_index());
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

    // FIXME: How best to display options here?
    std::panic!("Expected one of (options) at {}", self.get_pos_index());
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
          std::panic!("Unimplemented escape sequence at {}", self.get_pos_index());
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
      std::panic!(
        "Unexpected end of input after escape character at {}",
        self.get_pos_index(),
      );
    }

    return result;
  }

  fn assemble_function(&mut self) {
    self.parse_exact("function(");
    self.output.push(ValueType::Function as u8);

    self.fn_data = Default::default();

    self.fn_data.register_map.clear();
    self.fn_data.register_map.insert("return".to_string(), 0);
    self.fn_data.register_map.insert("this".to_string(), 1);
    self.fn_data.register_map.insert("ignore".to_string(), 0xff);

    loop {
      self.parse_optional_whitespace();
      let mut next = self.parse_one_of(&["%", ")"]);

      if next == ")" {
        self.fn_data.register_count_pos = self.output.len();
        self.output.push(0xff);
        self
          .output
          .push((self.fn_data.register_map.len() - 3) as u8); // TODO: Handle >255
        break;
      }

      if next != "%" {
        std::panic!("Expected this to be impossible");
      }

      let param_name = self.parse_identifier();

      if self.fn_data.register_map.contains_key(param_name.as_str()) {
        std::panic!(
          "Unexpected duplicate parameter name at {}",
          self.get_pos_index()
        );
      }

      self.get_register_index(param_name.as_str());
      self.parse_optional_whitespace();

      next = self.parse_one_of(&[",", ")"]);

      if next == ")" {
        self.fn_data.register_count_pos = self.output.len();
        self.output.push(0xff);
        self
          .output
          .push((self.fn_data.register_map.len() - 3) as u8); // TODO: Handle >255
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
        self.output.push(Instruction::End as u8);
        self.pos.next();
        break;
      }

      let optional_label = self.test_label();

      if optional_label.is_some() {
        self.assemble_label(optional_label.unwrap());
      } else {
        self.assemble_instruction();
      }
    }

    // TODO: Handle >255 registers
    self.output[self.fn_data.register_count_pos] = self.fn_data.register_map.len() as u8;

    self.fn_data.labels_map.resolve(&mut self.output);
  }

  fn assemble_class(&mut self) {
    self.parse_exact("class(");
    self.output.push(ValueType::Class as u8);
    self.parse_optional_whitespace();

    self.assemble_value();
    self.parse_optional_whitespace();

    self.parse_exact(",");
    self.parse_optional_whitespace();

    self.assemble_value();
    self.parse_optional_whitespace();

    self.parse_exact(")");
  }

  fn assemble_instruction(&mut self) {
    let instr = self.parse_instruction_word();

    self.output.push(instr.clone() as u8);

    for arg in get_instruction_layout(instr) {
      match arg {
        InstructionArg::Value => self.assemble_value(),
        InstructionArg::Register => self.assemble_register(),
        InstructionArg::Label => self.assemble_label_read(),
      }
    }
  }

  fn assemble_value(&mut self) {
    self.parse_optional_whitespace();

    match self.pos.peek() {
      None => std::panic!("Expected value at {}", self.get_pos_index()),
      Some('%') => {
        self.output.push(ValueType::Register as u8);
        self.assemble_register();
      }
      Some('@') => {
        self.parse_exact("@");
        self.output.push(ValueType::Pointer as u8);
        let definition_name = self.parse_identifier();
        self
          .definitions_map
          .add_unresolved(&definition_name, &mut self.output);
      }
      Some('$') => {
        self.parse_exact("$");
        self.output.push(ValueType::Builtin as u8);
        self.assemble_builtin();
      }
      Some('[') => {
        self.assemble_array();
      }
      Some('-' | '.' | '0'..='9') => {
        self.assemble_number();
      }
      Some('"') => {
        self.assemble_string();
      }
      Some('{') => {
        self.assemble_object();
      }
      Some(ref_c) => {
        let c = *ref_c;

        let parsed = self.parse_one_of(&["void", "undefined", "null", "false", "true", ""]);

        match parsed.as_str() {
          "void" => self.output.push(ValueType::Void as u8),
          "undefined" => self.output.push(ValueType::Undefined as u8),
          "null" => self.output.push(ValueType::Null as u8),
          "false" => self.output.push(ValueType::False as u8),
          "true" => self.output.push(ValueType::True as u8),

          // TODO: Finish implementing the different values
          _ => std::panic!(
            "Unimplemented value type or unexpected character {} at {}",
            c,
            self.get_pos_index(),
          ),
        }
      }
    }
  }

  fn assemble_array(&mut self) {
    self.parse_optional_whitespace();

    self.parse_exact("[");
    self.output.push(ValueType::Array as u8);

    loop {
      self.parse_optional_whitespace();

      match self.pos.peek() {
        None => std::panic!("Expected value or array end at {}", self.get_pos_index()),
        Some(']') => {
          self.pos.next();
          self.output.push(ValueType::End as u8);
          break;
        }
        _ => {}
      }

      self.assemble_value();
      self.parse_optional_whitespace();

      let next = self.parse_one_of(&[",", "]"]);

      if next == "," {
        self.pos.next(); // TODO: Assert whitespace
        continue;
      }

      if next == "]" {
        self.parse_optional_whitespace();
        self.output.push(ValueType::End as u8);
        break;
      }

      std::panic!("Expected this to be impossible");
    }
  }

  fn assemble_register(&mut self) {
    self.parse_optional_whitespace();
    self.parse_exact("%");
    let register_name = self.parse_identifier();
    let register_index = self.get_register_index(register_name.as_str());
    self.output.push(register_index);
  }

  fn assemble_builtin(&mut self) {
    match self.parse_one_of(&["Math", "Debug"]).as_str() {
      "Math" => self.write_varsize_uint(0),
      "Debug" => self.write_varsize_uint(1),
      _ => std::panic!("Shouldn't happen"),
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

  fn assemble_label(&mut self, label_name: String) {
    self.parse_optional_whitespace();

    advance_chars(&mut self.pos, label_name.len() + 1);

    self
      .fn_data
      .labels_map
      .found_locations
      .insert(label_name, self.output.len());
  }

  fn assemble_label_read(&mut self) {
    self.parse_optional_whitespace();
    self.parse_exact(":");
    let label_name = self.parse_identifier();
    self
      .fn_data
      .labels_map
      .add_unresolved(&label_name, &mut self.output);
  }

  fn assemble_number(&mut self) {
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
      std::panic!("Expected valid number at {}", self.get_pos_index());
    }

    let value = value_result.unwrap();

    if value == (value as i8) as f64 {
      self.output.push(ValueType::SignedByte as u8);

      for b in (value as i8).to_le_bytes() {
        self.output.push(b);
      }
    } else {
      self.output.push(ValueType::Number as u8);

      for b in value.to_le_bytes() {
        self.output.push(b);
      }
    }
  }

  fn assemble_string(&mut self) {
    let value = self.parse_string_literal();

    self.output.push(ValueType::String as u8);
    self.write_varsize_uint(value.len());

    for b in value.as_bytes() {
      self.output.push(*b);
    }
  }

  fn assemble_object(&mut self) {
    self.parse_exact("{");
    self.output.push(ValueType::Object as u8);

    loop {
      self.parse_optional_whitespace();
      let mut c = *self.pos.peek().expect("Expected object content or end");

      if c == '"' {
        self.assemble_string();
      } else if c == '%' {
        self.output.push(ValueType::Register as u8);
        self.assemble_register();
      } else if c == '@' {
        self.parse_exact("@");
        self.output.push(ValueType::Pointer as u8);
        let definition_name = self.parse_identifier();
        self
          .definitions_map
          .add_unresolved(&definition_name, &mut self.output);
      } else if c == '}' {
        self.output.push(ValueType::End as u8);
        self.pos.next();
        break;
      } else {
        std::panic!("Unexpected character {} at {}", c, self.get_pos_index());
      }

      self.parse_optional_whitespace();
      self.parse_exact(":");
      self.assemble_value();
      self.parse_optional_whitespace();

      c = *self.pos.peek().expect("Expected comma or object end");

      if c == ',' {
        self.pos.next();
      } else if c == '}' {
        self.output.push(ValueType::End as u8);
        self.pos.next();
        break;
      } else {
        std::panic!("Unexpected character {} at {}", c, self.get_pos_index());
      }
    }
  }

  fn get_register_index(&mut self, register_name: &str) -> u8 {
    let get_result = self.fn_data.register_map.get(&register_name.to_string());
    let result: u8;

    if get_result.is_none() {
      // TODO: Support >255 registers
      result = (self.fn_data.register_map.len() - 1) as u8;
      self
        .fn_data
        .register_map
        .insert(register_name.to_string(), result);
    } else {
      result = *get_result.unwrap();
    }

    return result;
  }

  fn write_varsize_uint(&mut self, value: usize) {
    let mut x = value;

    loop {
      let mut b: u8 = (x % 128) as u8;
      x /= 128;

      if x != 0 {
        b += 128;
      }

      self.output.push(b);

      if x == 0 {
        break;
      }
    }
  }
}

pub fn assemble(content: &str) -> Rc<Vec<u8>> {
  let mut assembler = Assembler {
    content: content,
    pos: content.chars().peekable(),
    output: Vec::new(),
    fn_data: Default::default(),
    definitions_map: LocationMap {
      references: HashMap::new(),
      found_locations: HashMap::new(),
    },
  };

  assembler.run();

  return Rc::new(assembler.output);
}

#[derive(Debug, Clone)]
enum Instruction {
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

enum InstructionArg {
  Value,
  Register,
  Label,
}

fn get_instruction_layout(instruction: Instruction) -> Vec<InstructionArg> {
  use Instruction::*;
  use InstructionArg::*;

  return match instruction {
    End => Vec::from([]),
    Mov => Vec::from([Value, Register]),
    OpInc => Vec::from([Register]),
    OpDec => Vec::from([Register]),
    OpPlus => Vec::from([Value, Value, Register]),
    OpMinus => Vec::from([Value, Value, Register]),
    OpMul => Vec::from([Value, Value, Register]),
    OpDiv => Vec::from([Value, Value, Register]),
    OpMod => Vec::from([Value, Value, Register]),
    OpExp => Vec::from([Value, Value, Register]),
    OpEq => Vec::from([Value, Value, Register]),
    OpNe => Vec::from([Value, Value, Register]),
    OpTripleEq => Vec::from([Value, Value, Register]),
    OpTripleNe => Vec::from([Value, Value, Register]),
    OpAnd => Vec::from([Value, Value, Register]),
    OpOr => Vec::from([Value, Value, Register]),
    OpNot => Vec::from([Value, Register]),
    OpLess => Vec::from([Value, Value, Register]),
    OpLessEq => Vec::from([Value, Value, Register]),
    OpGreater => Vec::from([Value, Value, Register]),
    OpGreaterEq => Vec::from([Value, Value, Register]),
    OpNullishCoalesce => Vec::from([Value, Value, Register]),
    OpOptionalChain => Vec::from([Value, Value, Register]),
    OpBitAnd => Vec::from([Value, Value, Register]),
    OpBitOr => Vec::from([Value, Value, Register]),
    OpBitNot => Vec::from([Value, Register]),
    OpBitXor => Vec::from([Value, Value, Register]),
    OpLeftShift => Vec::from([Value, Value, Register]),
    OpRightShift => Vec::from([Value, Value, Register]),
    OpRightShiftUnsigned => Vec::from([Value, Value, Register]),
    TypeOf => Vec::from([Value, Register]),
    InstanceOf => Vec::from([Value, Register]),
    In => Vec::from([Value, Value, Register]),
    Call => Vec::from([Value, Value, Register]),
    Apply => Vec::from([Value, Value, Value, Register]),
    Bind => Vec::from([Value, Value, Register]),
    Sub => Vec::from([Value, Value, Register]),
    SubMov => Vec::from([Value, Value, Register]),
    SubCall => Vec::from([Value, Value, Value, Register]),
    Jmp => Vec::from([Label]),
    JmpIf => Vec::from([Value, Label]),
    UnaryPlus => Vec::from([Value, Register]),
    UnaryMinus => Vec::from([Value, Register]),
    New => Vec::from([Value, Value, Register]),
  };
}

fn is_leading_identifier_char(c: char) -> bool {
  return c == '_' || ('a' <= c && c <= 'z') || ('A' <= c && c <= 'Z');
}

fn is_identifier_char(c: char) -> bool {
  return c == '_' || ('0' <= c && c <= '9') || ('a' <= c && c <= 'z') || ('A' <= c && c <= 'Z');
}

enum ValueType {
  End = 0x00,
  Void = 0x01,
  Undefined = 0x02,
  Null = 0x03,
  False = 0x04,
  True = 0x05,
  SignedByte = 0x06,
  Number = 0x07,
  String = 0x08,
  Array = 0x09,
  Object = 0x0a,
  Function = 0x0b,
  // Instance = 0x0c,
  Pointer = 0x0d,
  Register = 0x0e,
  // External = 0x0f,
  Builtin = 0x10,
  Class = 0x11,
}

fn advance_chars(iter: &mut std::iter::Peekable<std::str::Chars>, len: usize) {
  for _ in 0..len {
    iter.next();
  }
}
