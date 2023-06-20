use std::{cell::RefCell, rc::Rc};

use crate::vs_value::Val;

#[derive(Debug)]
pub struct CopyCounter {
  pub tag: Val,
  pub count: Rc<RefCell<usize>>,
}

impl CopyCounter {
  pub fn new(tag: Val) -> Self {
    CopyCounter {
      tag,
      count: Rc::new(RefCell::new(0)),
    }
  }
}

impl Clone for CopyCounter {
  fn clone(&self) -> Self {
    *self.count.borrow_mut() += 1;

    CopyCounter {
      tag: self.tag.clone(),
      count: self.count.clone(),
    }
  }
}
