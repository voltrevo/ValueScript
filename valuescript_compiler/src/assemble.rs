use std::rc::Rc;

use crate::assembler::assemble_module;
use crate::assembly_parser::parse_module;

pub fn assemble(content: &str) -> Rc<Vec<u8>> {
  let module = parse_module(content);
  let output = assemble_module(&module);

  // TODO: Don't use Rc
  return Rc::new(output);
}
