use std::process::exit;
use std::collections::HashMap;

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

fn assemble(content: &String) -> std::vec::Vec<u8> {
  let mut output: Vec<u8> = Vec::new();

  // TODO: Assemble content into output
  
  return output;
}

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

static instruction_word_map: HashMap<&str, Instruction> = HashMap::from([
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

fn parse_instruction_word(content: &String, pos: &usize) -> Instruction {
  for (word, instruction) in instruction_word_map {
    if test_instruction_word(content, *pos, word) {
      *pos += word.len() + 1;
      parse_optional_whitespace(content, pos);
      return instruction;
    }
  }

  std::panic!(std::format!("Failed to parse instruction at {}", pos));
}

fn test_chars(content: &str, pos: usize, chars: &str) -> bool {
  for c in chars.chars() {
    if pos >= content.len() || content.chars().nth(pos).unwrap() != c {
      return false;
    }

    pos += 1;
  }

  return true;
}

fn test_instruction_word(content: &str, pos: usize, word: &str) -> bool {
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

fn parse_optional_whitespace(content: &str, pos: &usize) {
  while *pos < content.len() {
    let c = content.chars().nth(*pos).unwrap();

    if c != ' ' && c != '\n' {
      return;
    }

    *pos += 1;
  }
}
