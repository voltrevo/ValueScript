use std::{cell::RefCell, fmt::Debug, rc::Rc};

use storage::StorageEntryPtr;

use crate::vs_value::{ToVal, Val};

#[derive(Debug)]
pub struct VsStoragePtr {
  pub ptr: StorageEntryPtr,
  cache: RefCell<Option<Val>>,
}

impl VsStoragePtr {
  pub fn from_ptr(ptr: StorageEntryPtr) -> Self {
    Self {
      ptr,
      cache: RefCell::new(None),
    }
  }

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

impl ToVal for VsStoragePtr {
  fn to_val(self) -> Val {
    Val::StoragePtr(Rc::new(self))
  }
}
