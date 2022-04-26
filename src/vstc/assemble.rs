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

fn assemble(content: &str) -> std::vec::Vec<u8> {
  let mut output: Vec<u8> = Vec::new();
  let mut pos: usize = 0;

  loop {
    parse_optional_whitespace(content, &mut pos);

    if pos >= content.len() {
      break;
    }

    assemble_definition(content, &mut pos, &mut output);
  }

  return output;
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

fn parse_instruction_word(content: &str, pos: &mut usize) -> Instruction {
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
    if test_instruction_word(content, *pos, word) {
      *pos += word.len() + 1;
      parse_optional_whitespace(content, pos);
      return instruction;
    }
  }

  std::panic!("Failed to parse instruction at {}", pos);
}

fn test_chars(content: &str, mut pos: usize, chars: &str) -> bool {
  for c in chars.chars() {
    if pos >= content.len() || content.chars().nth(pos).unwrap() != c {
      return false;
    }

    pos += 1;
  }

  return true;
}

fn test_instruction_word(content: &str, mut pos: usize, word: &str) -> bool {
  let has_chars = test_chars(content, pos, word);

  if !has_chars {
    return false;
  }

  pos += word.len();

  if pos >= content.len() {
    return true;
  }

  let ch = content.chars().nth(pos).unwrap();

  return ch == ' ' || ch == '\n';
}

fn parse_optional_whitespace(content: &str, pos: &mut usize) {
  while *pos < content.len() {
    let c = content.chars().nth(*pos).unwrap();

    if c != ' ' && c != '\n' {
      return;
    }

    *pos += 1;
  }
}

fn parse_identifier(content: &str, pos: &mut usize) -> String {
  let start = *pos;
  let leading_char = content.chars().nth(start).unwrap();

  if !is_leading_identifier_char(leading_char) {
    std::panic!("Invalid identifier at {}", pos);
  }

  *pos += 1;

  while *pos < content.len() {
    let c = content.chars().nth(*pos).unwrap();

    if !is_identifier_char(c) {
      break;
    }

    *pos += 1;
  }

  unsafe {
    return content.get_unchecked(start..*pos).to_string();
  }
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

fn parse_exact(content: &str, pos: &mut usize, chars: &str) {
  for c in chars.chars() {
    if *pos >= content.len() || content.chars().nth(*pos).unwrap() != c {
      std::panic!("Expected '{}' at {}", c, *pos);
    }

    *pos += 1;
  }
}

fn parse_optional_exact(content: &str, pos: &mut usize, chars: &str) -> bool {
  if test_chars(content, *pos, chars) {
    *pos += chars.len();
    return true;
  }

  return false;
}

fn parse_one_of(content: &str, pos: &mut usize, options: &[&str]) -> String {
  for opt in options {
    if test_chars(content, *pos, opt) {
      *pos += opt.len();
      return opt.to_string();
    }
  }

  // FIXME: How best to display options here?
  std::panic!("Expected one of (options) at {}", pos);
}

fn assemble_definition(content: &str, pos: &mut usize, output: &mut Vec<u8>) {
  parse_exact(content, pos, "@");
  let def_name = parse_identifier(content, pos);
  println!("assembling {}", def_name);
  parse_optional_whitespace(content, pos);
  parse_exact(content, pos, "=");
  parse_optional_whitespace(content, pos);

  // TODO: Handle other kinds of definitions
  assemble_function(content, pos, output);
}

fn assemble_function(content: &str, pos: &mut usize, output: &mut Vec<u8>) {
  parse_exact(content, pos, "function(");
  output.push(ValueType::Function as u8);

  let mut register_names: Vec<String> = Vec::from([
    "return".to_string(),
    "this".to_string(),
  ]);

  let mut param_names: HashSet<String> = HashSet::new();

  loop {
    parse_optional_whitespace(content, pos);
    let mut next = parse_one_of(content, pos, &["%", ")"]);

    if next == ")" {
      output.push(0xff); // TODO: This byte should be the number of registers
      output.push(param_names.len() as u8); // TODO: Handle >255 params
      break;
    }

    if next != "%" {
      std::panic!("Expected this to be impossible");
    }

    let param_name = parse_identifier(content, pos);
    param_names.insert(param_name.clone());
    register_names.push(param_name);
    parse_optional_whitespace(content, pos);

    next = parse_one_of(content, pos, &[",", ")"]);

    if next == ")" {
      output.push(0xff); // TODO: This byte should be the number of registers
      output.push(param_names.len() as u8); // TODO: Handle >255 params
      break;
    }
  }

  parse_optional_whitespace(content, pos);
  parse_exact(content, pos, "{");

  loop {
    parse_optional_whitespace(content, pos);

    let c = content.chars().nth(*pos);

    if c == None {
      std::panic!("Expected instruction or end of function at {}", pos);
    }

    if c.unwrap() == '}' {
      output.push(Instruction::End as u8);
      *pos += 1;
      break;
    }

    assemble_instruction(content, pos, output);
  }
}

enum ValueType {
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
}

fn assemble_instruction(content: &str, pos: &mut usize, output: &mut Vec<u8>) {
  let instr = parse_instruction_word(content, pos);
  println!("Skipping instruction {:?}", instr);
  skip_line(content, pos);
}

fn skip_line(content: &str, pos: &mut usize) {
  while *pos < content.len() {
    let c = content.chars().nth(*pos).unwrap();
    *pos += 1;

    if c == '\n' {
      return;
    }
  }

  std::panic!("Reached end of file looking for newline");
}
