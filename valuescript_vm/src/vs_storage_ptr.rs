use std::{
  cell::RefCell,
  fmt::{self, Debug},
  rc::Rc,
};

use storage::{StorageAutoPtr, StorageBackend, StorageEntity, StorageEntryPtr, StorageReader};

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
    let sb = match self.ptr.sb.upgrade() {
      Some(sb) => sb,
      None => panic!("Storage backend dropped"),
    };

    let borrow = sb.borrow();
    let read_cache = borrow.get_read_cache();

    if let Some(cache_box) = read_cache.get(&self.ptr.ptr.data) {
      if let Some(cache_val) = cache_box.downcast_ref::<Val>() {
        return cache_val.clone();
      }
    }

    drop(read_cache);

    let entry = sb.read(self.ptr.ptr).unwrap().expect("Unresolved ptr");
    let res = Val::from_storage_entry(&sb, entry).expect("Failed to deserialize Val");

    let mut read_cache = borrow.get_read_cache();

    read_cache.insert(self.ptr.ptr.data, Box::new(res.clone()));

    res
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

      // TODO: Since there's also caching in the storage backend's read_cache, is it worthwhile to
      // cache here too? (Perf test.)
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
