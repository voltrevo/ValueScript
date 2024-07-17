pub fn to_i32(x: f64) -> i32 {
  if x == f64::INFINITY {
    return 0;
  }

  let int1 = (x.trunc() as i64) & 0xffffffff;

  int1 as i32
}

pub fn to_u32(x: f64) -> u32 {
  if x == f64::INFINITY {
    return 0;
  }

  let int1 = (x.trunc() as i64) & 0xffffffff;

  int1 as u32
}

pub fn unicode_at(bytes: &[u8], len: usize, index: usize) -> Option<char> {
  code_point_at(bytes, len, index)
    .map(|code_point| std::char::from_u32(code_point).expect("Invalid code point"))
}

pub fn code_point_at(bytes: &[u8], len: usize, index: usize) -> Option<u32> {
  if index >= len {
    return None;
  }

  let byte = bytes[index];

  let leading_ones = byte.leading_ones() as usize;

  if leading_ones == 0 {
    return Some(byte as u32);
  }

  if leading_ones == 1 || leading_ones > 4 || index + leading_ones > len {
    return None;
  }

  let mut value = (byte & (0x7F >> leading_ones)) as u32;

  for i in 1..leading_ones {
    let next_byte = bytes[index + i];

    if next_byte.leading_ones() != 1 {
      return None;
    }

    value = (value << 6) | (next_byte & 0x3F) as u32;
  }

  Some(value)
}
