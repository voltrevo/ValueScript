use std::rc::Rc;
use std::collections::BTreeMap;

use super::vs_value::Val;
use super::vs_value::ValTrait;
use super::vs_pointer::VsPointer;
use super::vs_function::VsFunction;
use super::instruction::Instruction;
use super::vs_object::VsObject;
use super::vs_array::VsArray;
use super::vs_class::VsClass;
use super::builtins::get_builtin;

pub struct BytecodeDecoder {
  // TODO: Enable borrow usage to avoid the rc overhead
  pub data: Rc<Vec<u8>>,
  pub pos: usize,
}

#[repr(u8)]
#[derive(PartialEq)]
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
  Builtin = 0x10,
  Class = 0x11,
}

impl BytecodeType {
  fn from_byte(byte: u8) -> BytecodeType {
    use BytecodeType::*;

    return match byte {
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
      0x10 => Builtin,
      0x11 => Class,

      _ => std::panic!("Unrecognized BytecodeType"),
    };
  }
}

impl BytecodeDecoder {
  pub fn decode_byte(&mut self) -> u8 {
    let byte = self.data[self.pos];
    self.pos += 1;
    return byte;
  }

  pub fn peek_byte(&self) -> u8 {
    return self.data[self.pos];
  }

  pub fn decode_type(&mut self) -> BytecodeType {
    return BytecodeType::from_byte(self.decode_byte());
  }

  pub fn peek_type(&self) -> BytecodeType {
    return BytecodeType::from_byte(self.peek_byte());
  }

  pub fn decode_val(&mut self, registers: &Vec<Val>) -> Val {
    return match self.decode_type() {
      BytecodeType::End => std::panic!("Cannot decode end"),
      BytecodeType::Void => Val::Void,
      BytecodeType::Undefined => Val::Undefined,
      BytecodeType::Null => Val::Null,
      BytecodeType::False => Val::Bool(false),
      BytecodeType::True => Val::Bool(true),
      BytecodeType::SignedByte => Val::Number(
        self.decode_signed_byte() as f64
      ),
      BytecodeType::Number => Val::Number(
        self.decode_number()
      ),
      BytecodeType::String => Val::String(Rc::new(
        self.decode_string()
      )),
      BytecodeType::Array => {
        let mut vals: Vec<Val> = Vec::new();

        while self.peek_type() != BytecodeType::End {
          vals.push(self.decode_val(registers));
        }

        self.decode_type(); // End (TODO: assert)

        Val::Array(Rc::new(VsArray::from(vals)))
      },
      BytecodeType::Object => {
        let mut obj: BTreeMap<String, Val> = BTreeMap::new();

        while self.peek_type() != BytecodeType::End {
          obj.insert(
            self.decode_val(registers).val_to_string(),
            self.decode_val(registers),
          );
        }

        self.decode_type(); // End (TODO: assert)

        Val::Object(Rc::new(VsObject { string_map: obj, prototype: None }))
      },
      BytecodeType::Function => self.decode_function_header(),
      BytecodeType::Pointer => self.decode_pointer(),
      BytecodeType::Register => registers[self.decode_register_index().unwrap()].clone(),
      BytecodeType::Builtin => Val::Static(get_builtin(self.decode_varsize_uint())),
      BytecodeType::Class => Val::Class(Rc::new(VsClass {
        constructor: self.decode_val(registers),
        instance_prototype: self.decode_val(registers),
      }))
    }
  }

  pub fn decode_signed_byte(&mut self) -> i8 {
    let res = self.data[self.pos] as i8;
    self.pos += 1;
    return res;
  }

  pub fn decode_number(&mut self) -> f64 {
    let mut buf = [0u8; 8];
    let next_pos = self.pos + 8;
    buf.clone_from_slice(&self.data[self.pos..next_pos]);
    self.pos = next_pos;
    return f64::from_le_bytes(buf);
  }

  pub fn decode_string(&mut self) -> String {
    let len = self.decode_varsize_uint();
    let start = self.pos; // Start after decoding varsize
    let end = self.pos + len;
    let res = String::from_utf8_lossy(&self.data[start..end]).into_owned();
    self.pos = end;

    return res;
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
    return self.decode_byte() as usize + 256 * self.decode_byte() as usize;
  }

  pub fn decode_register_index(&mut self) -> Option<usize> {
    // TODO: Handle multi-byte registers
    let byte = self.decode_byte();

    if byte == 0xff {
      return None;
    }

    return Some(byte as usize);
  }

  pub fn clone_at(&self, pos: usize) -> BytecodeDecoder {
    return BytecodeDecoder { data: self.data.clone(), pos: pos };
  }

  pub fn decode_pointer(&mut self) -> Val {
    let from_pos = self.pos;
    let pos = self.decode_pos();
    
    if pos < from_pos {
      if
        self.clone_at(pos).decode_type() != BytecodeType::Function &&
        self.clone_at(pos).decode_type() != BytecodeType::Class
      {
        std::panic!("Invalid: non-function pointer that points backwards");
      }
    }

    return VsPointer::new(
      &self.data,
      pos,
    );
  }

  pub fn decode_function_header(&mut self) -> Val {
    // TODO: Support >256
    let register_count = self.decode_byte() as usize;
    let parameter_count = self.decode_byte() as usize;

    return Val::Function(Rc::new(VsFunction {
      bytecode: self.data.clone(),
      register_count: register_count,
      parameter_count: parameter_count,
      start: self.pos,
      binds: Vec::new(),
    }));
  }

  pub fn decode_instruction(&mut self) -> Instruction {
    return Instruction::from_byte(self.decode_byte());
  }
}
