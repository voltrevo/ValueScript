use rand::{rngs::ThreadRng, Rng};

use crate::storage::StorageEntry;

#[derive(serde::Serialize, serde::Deserialize, Hash, PartialEq, Eq)]
pub struct StoragePtr<T> {
  _marker: std::marker::PhantomData<T>,

  pub data: (u64, u64, u64),
}

impl<T> Clone for StoragePtr<T> {
  fn clone(&self) -> Self {
    Self {
      _marker: std::marker::PhantomData,
      data: self.data,
    }
  }
}

impl<T> Copy for StoragePtr<T> {}

impl<T> std::fmt::Debug for StoragePtr<T> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_tuple("StoragePtr")
      .field(&self.data.0)
      .field(&self.data.1)
      .field(&self.data.2)
      .finish()
  }
}

pub type StorageEntryPtr = StoragePtr<StorageEntry>;
pub type StorageHeadPtr = StoragePtr<StorageEntryPtr>;

impl<T> StoragePtr<T> {
  fn from_bytes(bytes: &[u8]) -> Self {
    if bytes.len() > 24 {
      panic!("Too many bytes");
    }

    let mut key = [0u8; 24];
    key[..bytes.len()].copy_from_slice(bytes);
    bincode::deserialize(&key).unwrap()
  }

  pub fn to_bytes(&self) -> Vec<u8> {
    bincode::serialize(self).unwrap()
  }

  pub fn random(rng: &mut ThreadRng) -> Self {
    Self {
      _marker: std::marker::PhantomData,
      data: (rng.gen(), rng.gen(), rng.gen()),
    }
  }
}

pub fn storage_head_ptr(name: &[u8]) -> StorageHeadPtr {
  StorageHeadPtr::from_bytes(name)
}

pub(crate) fn tmp_count_ptr() -> StoragePtr<u64> {
  let mut key = StoragePtr::<u64>::from_bytes(b"tmp");
  key.data.2 = 1;

  key
}

pub(crate) fn tmp_at_ptr(i: u64) -> StorageHeadPtr {
  let mut key = StorageHeadPtr::from_bytes(b"tmp");
  key.data.2 = i + 2;

  key
}
