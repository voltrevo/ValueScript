use std::{cell::RefCell, collections::HashMap, fmt, ops::Index, rc::Rc, slice::SliceIndex};

use crate::{bytecode_decoder::BytecodeDecoder, vs_value::Val};

pub struct Bytecode {
  pub code: Vec<u8>,
  pub cache: RefCell<HashMap<usize, Val>>,
}

impl<I: SliceIndex<[u8]>> Index<I> for Bytecode {
  type Output = I::Output;

  fn index(&self, index: I) -> &Self::Output {
    &self.code[index]
  }
}

impl fmt::Debug for Bytecode {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "Bytecode {{ code: {:?} }}", self.code)
  }
}

impl Bytecode {
  pub fn new(code: Vec<u8>) -> Bytecode {
    Bytecode {
      code,
      cache: RefCell::new(HashMap::new()),
    }
  }
}

pub trait DecoderMaker {
  fn decoder(&self, pos: usize) -> BytecodeDecoder;
}

impl DecoderMaker for Rc<Bytecode> {
  fn decoder(&self, pos: usize) -> BytecodeDecoder {
    BytecodeDecoder {
      bytecode: self.clone(),
      pos,
    }
  }
}
