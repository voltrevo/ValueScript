use std::rc::Rc;
use super::vs_value;

struct BytecodeDecoder {
  data: Rc<Vec<u8>>,
  pos: usize,
}

#[repr(u8)]
#[derive(PartialEq)]
enum BytecodeType {
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

      _ => std::panic!("Unrecognized BytecodeType"),
    };
  }

  pub fn decode_val(&mut self) -> vs_value::Val {
    use BytecodeType::*;

    return match self.decode_type() {
      SignedByte => vs_value::VsNumber::from_f64(
        self.decode_signed_byte() as f64
      ),
      Number => vs_value::VsNumber::from_f64(
        self.decode_number()
      ),
      String => vs_value::VsString::from_string(
        self.decode_string()
      ),
      _ => std::panic!("not imlemented"),
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
}
