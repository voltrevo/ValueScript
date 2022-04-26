use std::process::exit;
use std::collections::HashMap;
use std::str::FromStr;

pub fn command(args: &Vec<String>) {
  if args.len() != 3 {
    println!("ERROR: Unrecognized command\n");
    show_help();
    exit(1);
  }

  if args[2] == "-h" || args[2] == "--help" {
    show_help();
    return;
  }

  let read_result = std::fs::read_to_string(&args[2]);

  if read_result.is_err() {
    println!("Failed to read file {}", args[2]);
    return;
  }

  let content = read_result.expect("");
  let output_filename = "out.vsb";

  let write_result = std::fs::write(output_filename, assemble(&content));

  if write_result.is_err() {
    println!("Failed to write file {}", output_filename);
    std::process::exit(1);
  }
}

fn show_help() {
  println!("vstc assemble");
  println!("Convert ValueScript assembly to bytecode");
  println!("");
  println!("USAGE:");
  println!("    vstc assemble <file>");
}

struct AssemblerData {
  content: String, // TODO: Avoid copying this in
  pos: usize,
  output: Vec<u8>,
  register_map: HashMap<String, u8>,
  register_count_pos: usize,
  definitions_unresolved: HashMap<String, Vec<usize>>,
  definition_map: HashMap<String, u8>,
}

trait Assembler {
  fn run(&mut self);
  fn content_at(&self, pos: usize) -> char;
  fn test_chars(&self, chars: &str) -> bool;
  fn parse_optional_whitespace(&mut self);
  fn assemble_definition(&mut self);
  fn parse_instruction_word(&mut self) -> Instruction;
  fn test_instruction_word(&self, word: &str) -> bool;
  fn parse_identifier(&mut self) -> String;
  fn parse_exact(&mut self, chars: &str);
  fn parse_optional_exact(&mut self, chars: &str) -> bool;
  fn parse_one_of(&mut self, options: &[&str]) -> String;
  fn parse_string_literal(&mut self) -> String;
  fn assemble_function(&mut self);
  fn assemble_instruction(&mut self);
  fn assemble_value(&mut self);
  fn assemble_array(&mut self);
  fn assemble_register(&mut self);
  fn assemble_number(&mut self);
  fn assemble_string(&mut self);
  fn assemble_object(&mut self);
  fn get_register_index(&mut self, register_name: &str) -> u8;
  fn write_unresolved_definition(&mut self, definition_name: &str);
  fn write_varsize_uint(&mut self, value: usize);
  fn peek(&self, failure_msg: &str) -> char;
}

impl Assembler for AssemblerData {
  fn run(&mut self) {
    loop {
      self.parse_optional_whitespace();

      if self.pos >= self.content.len() {
        break;
      }

      self.assemble_definition();
    }

    for (def_name, locations) in &self.definitions_unresolved {
      let def_location_optional = self.definition_map.get(def_name);

      if def_location_optional.is_none() {
        std::panic!(
          "Unresolved reference to @{} at {}",
          def_name,
          locations[0],
        );
      }

      let def_location = def_location_optional.unwrap();

      for location in locations {
        self.output[*location] = *def_location;
      }
    }
  }

  fn content_at(&self, pos: usize) -> char {
    return self.content.chars().nth(pos).unwrap();
  }

  fn test_chars(&self, chars: &str) -> bool {
    let mut pos = self.pos;

    for c in chars.chars() {
      if pos >= self.content.len() || self.content_at(pos) != c {
        return false;
      }

      pos += 1;
    }

    return true;
  }

  fn parse_optional_whitespace(&mut self) {
    while self.pos < self.content.len() {
      let c = self.content_at(self.pos);

      if c != ' ' && c != '\n' {
        return;
      }

      self.pos += 1;
    }
  }

  fn assemble_definition(&mut self) {
    self.parse_exact("@");
    let def_name = self.parse_identifier();
    self.definition_map.insert(def_name, self.output.len() as u8); // TODO: Support >255
    self.parse_optional_whitespace();
    self.parse_exact("=");
    self.parse_optional_whitespace();

    // TODO: Handle other kinds of definitions
    self.assemble_function();
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
    ]);

    for (word, instruction) in instruction_word_map {
      if self.test_instruction_word(word) {
        self.pos += word.len() + 1;
        self.parse_optional_whitespace();
        return instruction;
      }
    }

    std::panic!("Failed to parse instruction at {}", self.pos);
  }

  fn test_instruction_word(&self, word: &str) -> bool {
    let mut pos = self.pos;
    let has_chars = self.test_chars(word);

    if !has_chars {
      return false;
    }

    pos += word.len();

    if pos >= self.content.len() {
      return true;
    }

    let ch = self.content_at(pos);

    return ch == ' ' || ch == '\n';
  }

  fn parse_identifier(&mut self) -> String {
    let start = self.pos;
    let leading_char = self.content_at(start);

    if !is_leading_identifier_char(leading_char) {
      std::panic!("Invalid identifier at {}", self.pos);
    }

    self.pos += 1;

    while self.pos < self.content.len() {
      let c = self.content_at(self.pos);

      if !is_identifier_char(c) {
        break;
      }

      self.pos += 1;
    }

    unsafe {
      return self.content.get_unchecked(start..self.pos).to_string();
    }
  }

  fn parse_exact(&mut self, chars: &str) {
    for c in chars.chars() {
      if self.pos >= self.content.len() || self.content_at(self.pos) != c {
        std::panic!("Expected '{}' at {}", c, self.pos);
      }

      self.pos += 1;
    }
  }

  fn parse_optional_exact(&mut self, chars: &str) -> bool {
    if self.test_chars(chars) {
      self.pos += chars.len();
      return true;
    }

    return false;
  }

  fn parse_one_of(&mut self, options: &[&str]) -> String {
    for opt in options {
      if self.test_chars(opt) {
        self.pos += opt.len();
        return opt.to_string();
      }
    }

    // FIXME: How best to display options here?
    std::panic!("Expected one of (options) at {}", self.pos);
  }

  fn parse_string_literal(&mut self) -> String {
    let mut result = "".to_string();

    self.parse_exact("\"");
    let mut escaping = false;

    while self.pos < self.content.len() {
      let c = self.content_at(self.pos);

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
          std::panic!("Unimplemented escape sequence at {}", self.pos);
        }

        escaping = false;
      } else if c == '\\' {
        escaping = true;
      } else if c == '"' {
        break;
      } else {
        result.push(c);
      }

      self.pos += 1;
    }

    if escaping {
      std::panic!(
        "Unexpected end of input after escape character at {}",
        self.pos,
      );
    }

    self.parse_exact("\"");

    return result;
  }

  fn assemble_function(&mut self) {
    self.parse_exact("function(");
    self.output.push(ValueType::Function as u8);

    self.register_map.clear();
    self.register_map.insert("return".to_string(), 0);
    self.register_map.insert("this".to_string(), 1);
    self.register_map.insert("ignore".to_string(), 0xff);

    loop {
      self.parse_optional_whitespace();
      let mut next = self.parse_one_of(&["%", ")"]);

      if next == ")" {
        self.register_count_pos = self.output.len();
        self.output.push(0xff);
        self.output.push((self.register_map.len() - 3) as u8); // TODO: Handle >255
        break;
      }

      if next != "%" {
        std::panic!("Expected this to be impossible");
      }

      let param_name = self.parse_identifier();

      if self.register_map.contains_key(param_name.as_str()) {
        std::panic!("Unexpected duplicate parameter name at {}", self.pos);
      }

      self.get_register_index(param_name.as_str());
      self.parse_optional_whitespace();

      next = self.parse_one_of(&[",", ")"]);

      if next == ")" {
        self.register_count_pos = self.output.len();
        self.output.push(0xff);
        self.output.push((self.register_map.len() - 3) as u8); // TODO: Handle >255
        break;
      }
    }

    self.parse_optional_whitespace();
    self.parse_exact("{");

    loop {
      self.parse_optional_whitespace();

      let c = self.content.chars().nth(self.pos);

      if c == None {
        std::panic!("Expected instruction or end of function at {}", self.pos);
      }

      if c.unwrap() == '}' {
        self.output.push(Instruction::End as u8);
        self.pos += 1;
        break;
      }

      self.assemble_instruction();
    }

    // TODO: Handle >255 registers
    self.output[self.register_count_pos] = self.register_map.len() as u8;
  }

  fn assemble_instruction(&mut self) {
    let instr = self.parse_instruction_word();

    self.output.push(instr.clone() as u8);
    
    for arg in get_instruction_layout(instr) {
      match arg {
        InstructionArg::Value => self.assemble_value(),
        InstructionArg::Register => self.assemble_register(),
      }
    }
  }

  fn assemble_value(&mut self) {
    self.parse_optional_whitespace();

    if self.pos >= self.content.len() {
      std::panic!("Expected value at {}", self.pos);
    }

    let c = self.content_at(self.pos);

    if c == '%' {
      self.output.push(ValueType::Register as u8);
      self.assemble_register();
    } else if c == '@' {
      self.parse_exact("@");
      self.output.push(ValueType::Pointer as u8);
      let definition_name = self.parse_identifier();
      self.write_unresolved_definition(definition_name.as_str());
    } else if c == '[' {
      self.assemble_array();
    } else if c == '-' || c == '.' || ('0' <= c && c <= '9') {
      self.assemble_number();
    } else if c == '"' {
      self.assemble_string();
    } else if c == '{' {
      self.assemble_object();
    } else {
      let parsed = self.parse_one_of(&[
        "void",
        "undefined",
        "null",
        "false",
        "true",
        "",
      ]);

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
          self.pos
        ),
      }
    }
  }

  fn assemble_array(&mut self) {
    self.parse_optional_whitespace();

    self.parse_exact("[");
    self.output.push(ValueType::Array as u8);

    loop {
      self.parse_optional_whitespace();

      if self.pos >= self.content.len() {
        std::panic!("Expected value or array end at {}", self.pos);
      }

      let c = self.content_at(self.pos);

      if c == ']' {
        self.pos += 1;
        self.output.push(ValueType::End as u8);
        break;
      }

      self.assemble_value();
      self.parse_optional_whitespace();

      let next = self.parse_one_of(&[",", "]"]);

      if next == "," {
        self.pos += 1;
        continue;
      }

      if next == "]" {
        self.pos += 1;
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

  fn assemble_number(&mut self) {
    let start = self.pos;

    while self.pos < self.content.len() {
      let c = self.content_at(self.pos);

      if c == '-' || c == '.' || c == 'e' || ('0' <= c && c <= '9') {
        self.pos += 1;
      } else {
        break;
      }
    }

    let value_result = f64::from_str(self.content.get(start..self.pos).unwrap());

    if value_result.is_err() {
      std::panic!("Expected valid number at {}", start);
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
      let mut c = self.peek("Expected object content or end");

      if c == '"' {
        self.assemble_string();
      } else if c == '%' {
        self.output.push(ValueType::Register as u8);
        self.assemble_register();
      } else if c == '@' {
        self.parse_exact("@");
        self.output.push(ValueType::Pointer as u8);
        let definition_name = self.parse_identifier();
        self.write_unresolved_definition(definition_name.as_str());
      } else if c == '}' {
        self.output.push(ValueType::End as u8);
        self.pos += 1;
        break;
      } else {
        std::panic!("Unexpected character {} at {}", c, self.pos);
      }

      self.parse_optional_whitespace();
      self.parse_exact(":");
      self.assemble_value();
      self.parse_optional_whitespace();

      c = self.peek("Expected comma or object end");

      if c == ',' {
        // Do nothing
      } else if c == '}' {
        self.output.push(ValueType::End as u8);
        self.pos += 1;
        break;
      } else {
        std::panic!("Unexpected character {} at {}", c, self.pos);
      }
    }
  }

  fn get_register_index(&mut self, register_name: &str) -> u8 {
    let get_result = self.register_map.get(&register_name.to_string());
    let result: u8;

    if get_result.is_none() {
      // TODO: Support >255 registers
      result = (self.register_map.len() - 1) as u8;
      self.register_map.insert(register_name.to_string(), result);
    } else {
      result = *get_result.unwrap();
    }

    return result;
  }

  fn write_unresolved_definition(&mut self, definition_name: &str) {
    if !self.definitions_unresolved.contains_key(definition_name) {
      self.definitions_unresolved.insert(
        definition_name.to_string(),
        Vec::new(),
      );
    }

    self.definitions_unresolved.get_mut(definition_name).unwrap()
      .push(self.output.len());
    
    self.output.push(0xff);
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

  fn peek(&self, failure_msg: &str) -> char {
    if self.pos >= self.content.len() {
      std::panic!("{} at {}", failure_msg, self.pos);
    }

    return self.content_at(self.pos);
  }
}

fn assemble(content: &str) -> Vec<u8> {
  let mut assembler = AssemblerData {
    content: content.to_string(),
    pos: 0,
    output: Vec::new(),
    register_map: HashMap::new(),
    register_count_pos: 0,
    definitions_unresolved: HashMap::new(),
    definition_map: HashMap::new(),
  };

  assembler.run();

  return assembler.output;
}

#[derive(Debug)]
#[derive(Clone)]
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
}

enum InstructionArg {
  Value,
  Register,
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
    OpBitNot => Vec::from([Value, Value, Register]),
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
    Jmp => Vec::from([Value]),
    JmpIf => Vec::from([Value, Value]),
  };
}

fn is_leading_identifier_char(c: char) -> bool {
  return
    c == '_' ||
    ('a' <= c && c <= 'z') ||
    ('A' <= c && c <= 'Z')
  ;
}

fn is_identifier_char(c: char) -> bool {
  return
    c == '_' ||
    ('0' <= c && c <= '9') ||
    ('a' <= c && c <= 'z') ||
    ('A' <= c && c <= 'Z')
  ;
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
  Instance = 0x0c,
  Pointer = 0x0d,
  Register = 0x0e,
  External = 0x0f,
}
