use std::{cell::RefCell, collections::HashMap, error::Error, rc::Weak};

use crate::{
  rc_key::RcKey,
  storage_backend::StorageError,
  storage_ptr::StorageEntryPtr,
  storage_tx::{StorageTx, StorageTxMut},
  StorageAutoPtr, StorageBackend, StorageEntity, StoragePtr,
};

#[derive(Default)]
pub struct MemoryBackend {
  data: HashMap<(u64, u64, u64), Vec<u8>>,
}

impl MemoryBackend {
  pub fn new() -> Self {
    Self {
      data: HashMap::new(),
    }
  }
}

impl StorageBackend for MemoryBackend {
  type CustomError = Box<dyn Error>;
  type Tx<'a> = MemoryTx<'a>;
  type TxMut<'a> = MemoryTxMut<'a>;

  fn transaction<F, T>(&self, self_weak: Weak<RefCell<Self>>, f: F) -> Result<T, Box<dyn Error>>
  where
    F: Fn(&mut Self::Tx<'_>) -> Result<T, StorageError<Self>>,
  {
    let mut handle = MemoryTx {
      backend: self_weak,
      storage: self,
    };

    f(&mut handle).map_err(|e| match e {
      StorageError::<MemoryBackend>::CustomError(e) => e,
      StorageError::<MemoryBackend>::Error(e) => e,
    })
  }

  fn transaction_mut<F, T>(
    &mut self,
    self_weak: Weak<RefCell<Self>>,
    f: F,
  ) -> Result<T, Box<dyn Error>>
  where
    F: Fn(&mut Self::TxMut<'_>) -> Result<T, StorageError<Self>>,
  {
    let mut handle = MemoryTxMut {
      backend: self_weak,
      ref_deltas: Default::default(),
      cache: Default::default(),
      storage: self,
    };

    let res = f(&mut handle).map_err(|e| match e {
      StorageError::<MemoryBackend>::CustomError(e) => e,
      StorageError::<MemoryBackend>::Error(e) => e,
    })?;

    handle.flush_ref_deltas().map_err(|e| match e {
      StorageError::<MemoryBackend>::CustomError(e) => e,
      StorageError::<MemoryBackend>::Error(e) => e,
    })?;

    Ok(res)
  }

  fn is_empty(&self) -> bool {
    self.data.is_empty()
  }

  #[cfg(test)]
  fn len(&self) -> usize {
    self.data.len()
  }
}

pub struct MemoryTx<'a> {
  backend: Weak<RefCell<MemoryBackend>>,
  storage: &'a MemoryBackend,
}

impl StorageTx<'_, MemoryBackend> for MemoryTx<'_> {
  fn read_bytes<T>(
    &self,
    ptr: StoragePtr<T>,
  ) -> Result<Option<Vec<u8>>, StorageError<MemoryBackend>> {
    Ok(self.storage.data.get(&ptr.data).cloned())
  }

  fn get_auto_ptr<SE: StorageEntity<MemoryBackend>>(
    &mut self,
    ptr: StorageEntryPtr,
  ) -> StorageAutoPtr<MemoryBackend, SE> {
    StorageAutoPtr {
      _marker: std::marker::PhantomData,
      sb: self.backend.clone(),
      ptr,
    }
  }
}

pub struct MemoryTxMut<'a> {
  backend: Weak<RefCell<MemoryBackend>>,
  ref_deltas: HashMap<(u64, u64, u64), i64>,
  cache: HashMap<RcKey, StorageEntryPtr>,
  storage: &'a mut MemoryBackend,
}

impl<'a> StorageTxMut<'a, MemoryBackend> for MemoryTxMut<'a> {
  fn ref_deltas(&mut self) -> &mut HashMap<(u64, u64, u64), i64> {
    &mut self.ref_deltas
  }

  fn cache(&mut self) -> &mut HashMap<RcKey, StorageEntryPtr> {
    &mut self.cache
  }

  fn read_bytes<T>(
    &self,
    ptr: StoragePtr<T>,
  ) -> Result<Option<Vec<u8>>, StorageError<MemoryBackend>> {
    Ok(self.storage.data.get(&ptr.data).cloned())
  }

  fn get_auto_ptr<SE: StorageEntity<MemoryBackend>>(
    &mut self,
    ptr: StorageEntryPtr,
  ) -> StorageAutoPtr<MemoryBackend, SE> {
    StorageAutoPtr {
      _marker: std::marker::PhantomData,
      sb: self.backend.clone(),
      ptr,
    }
  }

  fn write_bytes<T>(
    &mut self,
    ptr: StoragePtr<T>,
    data: Option<Vec<u8>>,
  ) -> Result<(), StorageError<MemoryBackend>> {
    match data {
      Some(data) => self.storage.data.insert(ptr.data, data),
      None => self.storage.data.remove(&ptr.data),
    };

    Ok(())
  }
}
