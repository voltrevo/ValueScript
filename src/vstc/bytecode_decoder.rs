use std::rc::Rc;
use super::vs_value;

struct BytecodeDecoder {
  data: Rc<Vec<u8>>,
  pos: usize,
}

#[derive(PartialEq)]
enum BytecodeType {
  Undefined = 0x02_u8,
  Null = 0x03_u8,
  False = 0x04_u8,
  True = 0x05_u8,
  SignedByte = 0x06_u8,
  Number = 0x07_u8,
  String = 0x08_u8,
  Array = 0x09_u8,
  Object = 0x0a_u8,
  Function = 0x0b_u8,
  Instance = 0x0c_u8,
}

impl BytecodeDecoder {
  pub fn decode_byte(&mut self) -> u8 {
    let byte = self.data[self.pos];
    self.pos += 1;
    return byte;    
  }

  pub fn decode_type(&mut self) -> BytecodeType {
    // TODO: Does this panic if byte is not a valid BytecodeType?
    return self.decode_byte() as BytecodeType;
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
      String => VsType::String,
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

  pub fn decode_string(&mut self) {
    let size = self.decode_varsize_uint();
    // TODO: Decode utf8 string
  }

  pub fn decode_varsize_uint(&mut self) -> usize {
    let res = 0_usize;

    loop {
      let byte = self.decode_byte();
      res += byte;

      if byte & 128 == 0 {
        return res;
      }

      res *= 128;
    }
  }
}
