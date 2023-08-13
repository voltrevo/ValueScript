use std::rc::Rc;

#[derive(Clone, Debug)]
pub struct VsFunctionMetadata {
  pub name: Rc<str>,
  pub hash: [u8; 32],
}
