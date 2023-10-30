use serde::{Deserialize, Serialize};

use crate::storage_ptr::StorageEntryPtr;

#[derive(Serialize, Deserialize)]
pub struct StorageEntry {
  pub(crate) ref_count: u64,
  pub(crate) refs: Vec<StorageEntryPtr>,
  pub(crate) data: Vec<u8>,
}

pub struct StorageEntryReader<'a> {
  pub entry: &'a StorageEntry,
  pub refs_i: usize,
  pub data_i: usize,
}

impl<'a> StorageEntryReader<'a> {
  pub fn new(entry: &'a StorageEntry) -> Self {
    Self {
      entry,
      refs_i: 0,
      data_i: 0,
    }
  }

  pub fn read_ref(&mut self) -> Option<StorageEntryPtr> {
    if self.refs_i >= self.entry.refs.len() {
      return None;
    }

    let ptr = self.entry.refs[self.refs_i];
    self.refs_i += 1;
    Some(ptr)
  }

  pub fn read_u8(&mut self) -> Option<u8> {
    if self.data_i >= self.entry.data.len() {
      return None;
    }

    let byte = self.entry.data[self.data_i];
    self.data_i += 1;
    Some(byte)
  }

  pub fn read_u64(&mut self) -> Option<u64> {
    if self.data_i + 8 > self.entry.data.len() {
      return None;
    }

    let bytes = self.entry.data.get(self.data_i..self.data_i + 8)?;
    self.data_i += 8;

    Some(u64::from_le_bytes(bytes.try_into().unwrap()))
  }
}
