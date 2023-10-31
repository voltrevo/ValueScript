use std::{collections::HashMap, error::Error};

use crate::{
  rc_key::RcKey, storage_ptr::StorageEntryPtr, storage_tx::StorageTx, StorageBackend, StoragePtr,
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
  type InTxError = Box<dyn Error>;
  type Tx<'a> = MemoryTx<'a>;

  fn transaction<F, T>(&mut self, f: F) -> Result<T, Box<dyn Error>>
  where
    F: Fn(&mut Self::Tx<'_>) -> Result<T, Self::InTxError>,
  {
    let mut handle = MemoryTx {
      ref_deltas: Default::default(),
      cache: Default::default(),
      storage: self,
    };

    let res = f(&mut handle)?;
    handle.flush_ref_deltas()?;

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
  ref_deltas: HashMap<(u64, u64, u64), i64>,
  cache: HashMap<RcKey, StorageEntryPtr>,
  storage: &'a mut MemoryBackend,
}

impl<'a> StorageTx<'a, MemoryBackend> for MemoryTx<'a> {
  fn ref_deltas(&mut self) -> &mut HashMap<(u64, u64, u64), i64> {
    &mut self.ref_deltas
  }

  fn cache(&mut self) -> &mut HashMap<RcKey, StorageEntryPtr> {
    &mut self.cache
  }

  fn read_bytes<T>(&self, ptr: StoragePtr<T>) -> Result<Option<Vec<u8>>, Box<dyn Error>> {
    Ok(self.storage.data.get(&ptr.data).cloned())
  }

  fn write_bytes<T>(
    &mut self,
    ptr: StoragePtr<T>,
    data: Option<Vec<u8>>,
  ) -> Result<(), Box<dyn Error>> {
    match data {
      Some(data) => self.storage.data.insert(ptr.data, data),
      None => self.storage.data.remove(&ptr.data),
    };

    Ok(())
  }
}
