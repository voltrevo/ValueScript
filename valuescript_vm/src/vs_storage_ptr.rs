use std::{
  cell::RefCell,
  fmt::{self, Debug},
  rc::Rc,
};

use storage::{StorageAutoPtr, StorageBackend, StorageEntryPtr};

use crate::vs_value::{ToVal, Val};

pub trait ValResolver: Debug {
  fn resolve(&self) -> Val;
  fn ptr(&self) -> StorageEntryPtr;
}

pub struct StorageValAutoPtr<SB: StorageBackend + 'static> {
  ptr: StorageAutoPtr<SB, Val>,
}

impl<SB: StorageBackend> ValResolver for StorageValAutoPtr<SB> {
  fn resolve(&self) -> Val {
    self.ptr.resolve().unwrap().unwrap()
  }

  fn ptr(&self) -> StorageEntryPtr {
    self.ptr.ptr
  }
}

impl<SB: StorageBackend> Debug for StorageValAutoPtr<SB> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("StorageValAutoPtr")
      .field("ptr", &self.ptr)
      .finish()
  }
}

#[derive(Debug)]
pub struct VsStoragePtr {
  pub(crate) resolver: Box<dyn ValResolver>,
  cache: RefCell<Option<Val>>,
}

impl VsStoragePtr {
  pub fn new<SB: StorageBackend + 'static>(auto_ptr: StorageAutoPtr<SB, Val>) -> Self {
    Self {
      resolver: Box::new(StorageValAutoPtr { ptr: auto_ptr }),
      cache: RefCell::new(None),
    }
  }

  pub fn get(&self) -> Val {
    let mut cache = self.cache.borrow_mut();

    if let Some(val) = &*cache {
      return val.clone();
    }

    let val = self.resolver.resolve();
    *cache = Some(val.clone());

    val
  }
}

impl ToVal for VsStoragePtr {
  fn to_val(self) -> Val {
    Val::StoragePtr(Rc::new(self))
  }
}
