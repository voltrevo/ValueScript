use std::process::exit;
use std::collections::HashMap;
use std::collections::HashSet;

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
  fn assemble_function(&mut self);
  fn assemble_instruction(&mut self);
  fn assemble_value(&mut self);
  fn assemble_array(&mut self);
  fn assemble_register(&mut self);
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
    println!("assembling {}", def_name);
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

  fn assemble_function(&mut self) {
    self.parse_exact("function(");
    self.output.push(ValueType::Function as u8);

    let mut register_names: Vec<String> = Vec::from([
      "return".to_string(),
      "this".to_string(),
    ]);

    let mut param_names: HashSet<String> = HashSet::new();

    loop {
      self.parse_optional_whitespace();
      let mut next = self.parse_one_of(&["%", ")"]);

      if next == ")" {
        self.output.push(0xff); // TODO: This byte should be the number of registers
        self.output.push(param_names.len() as u8); // TODO: Handle >255 params
        break;
      }

      if next != "%" {
        std::panic!("Expected this to be impossible");
      }

      let param_name = self.parse_identifier();
      param_names.insert(param_name.clone());
      register_names.push(param_name);
      self.parse_optional_whitespace();

      next = self.parse_one_of(&[",", ")"]);

      if next == ")" {
        self.output.push(0xff); // TODO: This byte should be the number of registers
        self.output.push(param_names.len() as u8); // TODO: Handle >255 params
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
  }

  fn assemble_instruction(&mut self) {
    let instr = self.parse_instruction_word();
    
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
      self.parse_identifier();

      // TODO: Definition location based on identifier needs to be written here
      self.output.push(0xff);
    } else if c == '[' {
      self.assemble_array();
    } else {
      std::panic!("Unexpected character {} at {}", c, self.pos);
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
    self.parse_identifier();

    // TODO: Register number based on identifier needs to be written here
    self.output.push(0xff);
  }
}

fn assemble(content: &str) -> Vec<u8> {
  let mut assembler = AssemblerData {
    content: content.to_string(),
    pos: 0,
    output: Vec::new(),
  };

  assembler.run();

  return assembler.output;
}

#[derive(Debug)]
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
