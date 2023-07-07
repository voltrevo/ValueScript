use std::collections::BTreeMap;
use std::mem::take;
use std::rc::Rc;

use num_bigint::BigInt;
use num_bigint::Sign;
use valuescript_common::InstructionByte;

use crate::builtins::BUILTIN_VALS;
use crate::bytecode::Bytecode;
use crate::vs_class::VsClass;
use crate::vs_function::VsFunction;
use crate::vs_object::VsObject;
use crate::vs_symbol::VsSymbol;
use crate::vs_value::ToVal;
use crate::vs_value::Val;

#[derive(Clone)]
pub struct BytecodeDecoder {
  // TODO: Enable borrow usage to avoid the rc overhead
  pub bytecode: Rc<Bytecode>,
  pub pos: usize,
}

#[repr(u8)]
#[derive(PartialEq, Debug)]
pub enum BytecodeType {
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
  Pointer = 0x0d,
  Register = 0x0e,
  TakeRegister = 0x0f,
  Builtin = 0x10,
  Class = 0x11,
  BigInt = 0x13,
  GeneratorFunction = 0x14,
  Unrecognized = 0xff,
}

impl BytecodeType {
  fn from_byte(byte: u8) -> BytecodeType {
    use BytecodeType::*;

    match byte {
      0x00 => End,
      0x01 => Void,
      0x02 => Undefined,
      0x03 => Null,
      0x04 => False,
      0x05 => True,
      0x06 => SignedByte,
      0x07 => Number,
      0x08 => String,
      0x09 => Array,
      0x0a => Object,
      0x0b => Function,
      0x0d => Pointer,
      0x0e => Register,
      0x0f => TakeRegister,
      0x10 => Builtin,
      0x11 => Class,

      0x13 => BigInt,
      0x14 => GeneratorFunction,

      _ => Unrecognized,
    }
  }
}

impl BytecodeDecoder {
  pub fn decode_byte(&mut self) -> u8 {
    let byte = self.bytecode[self.pos];
    self.pos += 1;
    byte
  }

  pub fn peek_byte(&self) -> u8 {
    self.bytecode[self.pos]
  }

  pub fn decode_type(&mut self) -> BytecodeType {
    BytecodeType::from_byte(self.decode_byte())
  }

  pub fn peek_type(&self) -> BytecodeType {
    BytecodeType::from_byte(self.peek_byte())
  }

  pub fn decode_val(&mut self, registers: &mut Vec<Val>) -> Val {
    return match self.decode_type() {
      BytecodeType::End => panic!("Cannot decode end"),
      BytecodeType::Void => Val::Void,
      BytecodeType::Undefined => Val::Undefined,
      BytecodeType::Null => Val::Null,
      BytecodeType::False => Val::Bool(false),
      BytecodeType::True => Val::Bool(true),
      BytecodeType::SignedByte => (self.decode_signed_byte() as f64).to_val(),
      BytecodeType::Number => self.decode_number().to_val(),
      BytecodeType::String => self.decode_string().to_val(),
      BytecodeType::Array => self.decode_vec_val(registers).to_val(),
      BytecodeType::Object => {
        let mut string_map: BTreeMap<String, Val> = BTreeMap::new();
        let mut symbol_map: BTreeMap<VsSymbol, Val> = BTreeMap::new();

        while self.peek_type() != BytecodeType::End {
          let key = self.decode_val(registers);
          let value = self.decode_val(registers);

          match key {
            Val::String(string) => string_map.insert(string.to_string(), value),
            Val::Symbol(symbol) => symbol_map.insert(symbol, value),
            key => string_map.insert(key.to_string(), value),
          };
        }

        self.decode_type(); // End (TODO: assert)

        VsObject {
          string_map,
          symbol_map,
          prototype: None,
        }
        .to_val()
      }
      BytecodeType::Function => self.decode_function(false),
      BytecodeType::Pointer => self.decode_pointer(registers),
      BytecodeType::Register => match registers[self.decode_register_index().unwrap()].clone() {
        Val::Void => Val::Undefined,
        val => val,
      },
      BytecodeType::TakeRegister => match &mut registers[self.decode_register_index().unwrap()] {
        Val::Void => Val::Undefined,
        val => take(val),
      },
      BytecodeType::Builtin => BUILTIN_VALS[self.decode_varsize_uint()](),
      BytecodeType::Class => VsClass {
        constructor: self.decode_val(registers),
        prototype: self.decode_val(registers),
        static_: self.decode_val(registers),
      }
      .to_val(),
      BytecodeType::BigInt => self.decode_bigint().to_val(),
      BytecodeType::GeneratorFunction => self.decode_function(true),
      BytecodeType::Unrecognized => panic!("Unrecognized bytecode type at {}", self.pos - 1),
    };
  }

  pub fn decode_vec_val(&mut self, registers: &mut Vec<Val>) -> Vec<Val> {
    let mut vals: Vec<Val> = Vec::new();

    while self.peek_type() != BytecodeType::End {
      vals.push(self.decode_val(registers));
    }

    self.decode_type(); // End (TODO: assert)

    vals
  }

  pub fn decode_signed_byte(&mut self) -> i8 {
    let res = self.bytecode[self.pos] as i8;
    self.pos += 1;
    res
  }

  pub fn decode_number(&mut self) -> f64 {
    let mut buf = [0u8; 8];
    let next_pos = self.pos + 8;
    buf.clone_from_slice(&self.bytecode[self.pos..next_pos]);
    self.pos = next_pos;
    f64::from_le_bytes(buf)
  }

  pub fn decode_bigint(&mut self) -> BigInt {
    let sign = match self.decode_byte() {
      0 => Sign::Minus,
      1 => Sign::NoSign,
      2 => Sign::Plus,

      _ => panic!("Invalid sign for bigint"),
    };

    let len = self.decode_varsize_uint();
    let bytes = &self.bytecode[self.pos..self.pos + len];
    self.pos += len;

    BigInt::from_bytes_le(sign, bytes)
  }

  pub fn decode_string(&mut self) -> String {
    let len = self.decode_varsize_uint();
    let start = self.pos; // Start after decoding varsize
    let end = self.pos + len;
    let res = String::from_utf8_lossy(&self.bytecode[start..end]).into_owned();
    self.pos = end;

    res
  }

  pub fn decode_varsize_uint(&mut self) -> usize {
    let mut res = 0_usize;
    let mut mul = 1_usize;

    loop {
      let byte = self.decode_byte();
      res += mul * ((byte % 128) as usize);

      if byte & 128 == 0 {
        return res;
      }

      mul *= 128;
    }
  }

  pub fn decode_pos(&mut self) -> usize {
    // TODO: the number of bytes to represent a position should be based on the
    // size of the bytecode
    self.decode_byte() as usize + 256 * self.decode_byte() as usize
  }

  pub fn decode_register_index(&mut self) -> Option<usize> {
    // TODO: Handle multi-byte registers
    let byte = self.decode_byte();

    if byte == 0xff {
      return None;
    }

    Some(byte as usize)
  }

  pub fn clone_at(&self, pos: usize) -> BytecodeDecoder {
    BytecodeDecoder {
      bytecode: self.bytecode.clone(),
      pos,
    }
  }

  pub fn decode_pointer(&mut self, registers: &mut Vec<Val>) -> Val {
    let from_pos = self.pos;
    let pos = self.decode_pos();

    if pos < from_pos {
      // Question: Why is this different from self.peek_type()?
      let type_ = self.clone_at(pos).decode_type();

      match type_ {
        BytecodeType::Function
        | BytecodeType::GeneratorFunction
        | BytecodeType::Class
        | BytecodeType::Unrecognized => {}
        _ => {
          panic!("Invalid: {:?} pointer that points backwards", type_);
        }
      }
    }

    let cached_val = self.bytecode.cache.borrow().get(&pos).cloned();

    match cached_val {
      Some(val) => val,
      None => {
        let val = self.clone_at(pos).decode_val(registers);
        self.bytecode.cache.borrow_mut().insert(pos, val.clone());

        val
      }
    }
  }

  pub fn decode_function(&mut self, is_generator: bool) -> Val {
    // TODO: Support >256
    let register_count = self.decode_byte() as usize;
    let parameter_count = self.decode_byte() as usize;

    VsFunction {
      bytecode: self.bytecode.clone(),
      is_generator,
      register_count,
      parameter_count,
      start: self.pos,
      binds: Vec::new(),
    }
    .to_val()
  }

  pub fn decode_instruction(&mut self) -> InstructionByte {
    InstructionByte::from_byte(self.decode_byte())
  }
}
