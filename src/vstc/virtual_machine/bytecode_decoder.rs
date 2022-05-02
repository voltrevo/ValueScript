use std::rc::Rc;

use super::vs_value::Val;
use super::vs_number::VsNumber;
use super::vs_string::VsString;
use super::vs_pointer::VsPointer;

pub struct BytecodeDecoder {
  // TODO: Enable borrow usage to avoid the rc overhead
  pub data: Rc<Vec<u8>>,
  pub pos: usize,
}

#[repr(u8)]
#[derive(PartialEq)]
pub enum BytecodeType {
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
}

impl BytecodeDecoder {
  pub fn decode_byte(&mut self) -> u8 {
    let byte = self.data[self.pos];
    self.pos += 1;
    return byte;
  }

  pub fn decode_type(&mut self) -> BytecodeType {
    use BytecodeType::*;

    return match self.decode_byte() {
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
      0x0c => Instance,
      0x0d => Pointer,
      0x0e => Register,

      _ => std::panic!("Unrecognized BytecodeType"),
    };
  }

  pub fn decode_val(&mut self) -> Val {
    use BytecodeType::*;

    return match self.decode_type() {
      Undefined => std::panic!("Not implemented"),
      Null => std::panic!("Not implemented"),
      False => std::panic!("Not implemented"),
      True => std::panic!("Not implemented"),
      SignedByte => VsNumber::from_f64(
        self.decode_signed_byte() as f64
      ),
      Number => VsNumber::from_f64(
        self.decode_number()
      ),
      String => VsString::from_string(
        self.decode_string()
      ),
      Array => std::panic!("Not implemented"),
      Object => std::panic!("Not implemented"),
      Function => std::panic!("Not implemented"),
      Instance => std::panic!("Not implemented"),
      Pointer => {
        let from_pos = self.pos;
        let pos = self.decode_pos();
        
        if pos < from_pos {
          if self.clone_at(pos).decode_type() != BytecodeType::Function {
            std::panic!("Invalid: non-function pointer that points backwards");
          }
        }

        return VsPointer::new(
          &self.data,
          pos,
        );
      },
      Register => std::panic!("Not implemented"),
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
    let start = self.pos;
    self.pos += self.decode_varsize_uint();
    let res = String::from_utf8_lossy(&self.data[start..self.pos]).into_owned();

    return res;
  }

  pub fn decode_varsize_uint(&mut self) -> usize {
    let mut res = 0_usize;

    loop {
      let byte = self.decode_byte();
      res += byte as usize;

      if byte & 128 == 0 {
        return res;
      }

      res *= 128;
    }
  }

  pub fn decode_pos(&mut self) -> usize {
    // TODO: the number of bytes to represent a position should be based on the
    // size of the bytecode
    return self.decode_byte() as usize;
  }

  pub fn clone_at(&self, pos: usize) -> BytecodeDecoder {
    return BytecodeDecoder { data: self.data.clone(), pos: pos };
  }
}
