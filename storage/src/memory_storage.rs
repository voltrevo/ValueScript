use std::collections::HashMap;

use crate::storage::{RawStorage, StorageKey};

pub struct MemoryStorage {
  data: HashMap<StorageKey, Vec<u8>>,
}

impl MemoryStorage {
  pub fn new() -> Self {
    Self {
      data: HashMap::new(),
    }
  }
}

impl RawStorage for MemoryStorage {
  fn read(&self, key: StorageKey) -> Option<Vec<u8>> {
    self.data.get(&key).cloned()
  }

  fn write(&mut self, key: StorageKey, data: Option<Vec<u8>>) {
    match data {
      Some(data) => self.data.insert(key, data),
      None => self.data.remove(&key),
    };
  }
}
