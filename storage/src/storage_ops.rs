use rand::thread_rng;
use serde::{Deserialize, Serialize};

use crate::{
  storage::StorageVal,
  storage_backend::StorageBackendHandle,
  storage_ptr::{StorageEntryPtr, StorageHeadPtr, StoragePtr},
};

pub trait StorageOps<E> {
  fn read<T: for<'de> Deserialize<'de>>(&mut self, key: StoragePtr<T>) -> Result<Option<T>, E>;
  fn write<T: Serialize>(&mut self, key: StoragePtr<T>, data: Option<&T>) -> Result<(), E>;

  fn inc_ref(&mut self, key: StorageEntryPtr) -> Result<(), E>;
  fn dec_ref(&mut self, key: StorageEntryPtr) -> Result<(), E>;

  fn get_head(&mut self, ptr: StorageHeadPtr) -> Result<Option<StorageVal>, E>;
  fn set_head(&mut self, ptr: StorageHeadPtr, value: Option<&StorageVal>) -> Result<(), E>;

  fn store_with_replacements(&mut self, value: &StorageVal) -> Result<StorageEntryPtr, E>;
  fn store(&mut self, value: &StorageVal) -> Result<StorageEntryPtr, E>;
}

impl<'a, Handle, E> StorageOps<E> for Handle
where
  Handle: StorageBackendHandle<'a, E>,
{
  fn read<T: for<'de> Deserialize<'de>>(&mut self, key: StoragePtr<T>) -> Result<Option<T>, E> {
    let data = match self.read_bytes(key)? {
      Some(data) => data,
      None => return Ok(None),
    };

    Ok(Some(bincode::deserialize(&data).unwrap()))
  }

  fn write<T: Serialize>(&mut self, key: StoragePtr<T>, data: Option<&T>) -> Result<(), E> {
    self.write_bytes(key, data.map(|data| bincode::serialize(&data).unwrap()))
  }

  fn inc_ref(&mut self, key: StorageEntryPtr) -> Result<(), E> {
    let mut entry = match self.read(key)? {
      Some(entry) => entry,
      None => panic!("Key does not exist"),
    };

    entry.ref_count += 1;

    self.write(key, Some(&entry))
  }

  fn dec_ref(&mut self, key: StorageEntryPtr) -> Result<(), E> {
    let mut entry = match self.read(key)? {
      Some(entry) => entry,
      None => panic!("Key does not exist"),
    };

    entry.ref_count -= 1;

    if entry.ref_count == 0 {
      for key in entry.refs.iter() {
        self.dec_ref(*key)?;
      }

      self.write(key, None)
    } else {
      self.write(key, Some(&entry))
    }
  }

  fn get_head(&mut self, ptr: StorageHeadPtr) -> Result<Option<StorageVal>, E> {
    let key = match self.read(ptr)? {
      Some(key) => key,
      None => return Ok(None),
    };

    let value = self.read(key)?.map(|entry| entry.to_val());

    Ok(value)
  }

  fn set_head(&mut self, ptr: StorageHeadPtr, value: Option<&StorageVal>) -> Result<(), E> {
    if let Some(value) = value {
      let key = self.store_with_replacements(value)?;

      // TODO: Performance: Identify overlapping keys and cancel out inc+dec
      if let Some(old_key) = self.read(ptr)? {
        self.dec_ref(old_key)?;
      }

      self.write(ptr, Some(&key))
    } else {
      if let Some(old_key) = self.read(ptr)? {
        self.dec_ref(old_key)?;
      }

      self.write(ptr, None)
    }
  }

  fn store_with_replacements(&mut self, value: &StorageVal) -> Result<StorageEntryPtr, E> {
    if let Some(key) = value.point.maybe_replace_store(self, &value.refs)? {
      return Ok(key);
    }

    self.store(value)
  }

  fn store(&mut self, value: &StorageVal) -> Result<StorageEntryPtr, E> {
    let key = StoragePtr::random(&mut thread_rng());
    self.write(key, Some(&value.to_entry()))?;

    for subkey in value.refs.iter() {
      self.inc_ref(*subkey)?;
    }

    Ok(key)
  }
}
