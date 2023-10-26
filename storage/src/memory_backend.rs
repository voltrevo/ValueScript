use std::collections::HashMap;
use std::fmt::Debug as DebugTrait;

use crate::storage::{StorageBackend, StorageBackendHandle, StorageKey};

pub struct MemoryBackend {
  data: HashMap<StorageKey, Vec<u8>>,
}

impl MemoryBackend {
  pub fn new() -> Self {
    Self {
      data: HashMap::new(),
    }
  }
}

impl StorageBackend for MemoryBackend {
  type Error<E: DebugTrait> = E;
  type InTransactionError<E> = E;
  type Handle<'a, E> = MemoryStorageHandle<'a>;

  fn transaction<F, T, E: DebugTrait>(&mut self, f: F) -> Result<T, Self::Error<E>>
  where
    F: Fn(&mut Self::Handle<'_, E>) -> Result<T, Self::InTransactionError<E>>,
  {
    let mut handle = MemoryStorageHandle { storage: self };
    f(&mut handle)
  }
}

pub struct MemoryStorageHandle<'a> {
  storage: &'a mut MemoryBackend,
}

impl<'a, E> StorageBackendHandle<'a, E> for MemoryStorageHandle<'a> {
  fn read(&self, key: StorageKey) -> Result<Option<Vec<u8>>, E> {
    Ok(self.storage.data.get(&key).cloned())
  }

  fn write(&mut self, key: StorageKey, data: Option<Vec<u8>>) -> Result<(), E> {
    match data {
      Some(data) => self.storage.data.insert(key, data),
      None => self.storage.data.remove(&key),
    };

    Ok(())
  }
}
