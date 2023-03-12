use crate::vs_value::{Val, ValTrait};

pub fn to_wrapping_index(index: Option<&Val>, len: usize) -> Option<usize> {
  let unchecked = match index {
    None => {
      return None;
    }
    Some(i) => to_unchecked_wrapping_index(i, len),
  };

  if unchecked < 0 || unchecked as usize >= len {
    return None;
  }

  return Some(unchecked as usize);
}

pub fn to_unchecked_wrapping_index(index: &Val, len: usize) -> isize {
  let index_num = index.to_number();

  let mut floored_index = index_num.trunc();
  let f64_len = len as f64;

  if floored_index < 0_f64 {
    floored_index += f64_len;
  }

  // TODO: Investigate potential pitfalls for arrays with length exceeding max
  // isize.
  return floored_index as isize;
}
