use std::{cell::RefCell, fmt::Debug};

use storage::StorageEntryPtr;

use crate::vs_value::Val;

#[derive(Debug)]
pub struct VsStoragePtr {
  ptr: StorageEntryPtr,
  cache: RefCell<Option<Val>>,
}

impl VsStoragePtr {
  pub fn get(&self) -> Val {
    #[allow(unused_mut)] // Used in commented code
    let mut cache = self.cache.borrow_mut();

    if let Some(val) = &*cache {
      return val.clone();
    }

    todo!()
    // let val = /* TODO */;
    // *cache = Some(val.clone());
    // val
  }
}
