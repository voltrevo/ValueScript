use std::rc::Rc;

use valuescript_vm::{vs_value::Val, Bytecode, DecoderMaker};

use crate::{assemble, compile_str};

pub fn inline_valuescript(source: &str) -> Val {
  Rc::new(Bytecode::new(assemble(
    &compile_str(source).module.unwrap(),
  )))
  .decoder(0)
  .decode_val(&mut vec![])
}
