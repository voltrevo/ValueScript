use std::mem::take;

use rand::thread_rng;
use serde::{Deserialize, Serialize};

use crate::{
  rc_key::RcKey,
  storage_backend::StorageBackendHandle,
  storage_ptr::{StorageEntryPtr, StorageHeadPtr, StoragePtr},
  storage_val::StorageVal,
};

pub trait StorageOps<E> {
  fn read<T: for<'de> Deserialize<'de>>(&mut self, key: StoragePtr<T>) -> Result<Option<T>, E>;
  fn write<T: Serialize>(&mut self, key: StoragePtr<T>, data: Option<&T>) -> Result<(), E>;

  fn get_head(&mut self, ptr: StorageHeadPtr) -> Result<Option<StorageVal>, E>;
  fn set_head(&mut self, ptr: StorageHeadPtr, value: Option<&StorageVal>) -> Result<(), E>;

  fn store_with_replacements(&mut self, value: &StorageVal) -> Result<StorageEntryPtr, E>;
  fn store(&mut self, value: &StorageVal) -> Result<StorageEntryPtr, E>;

  fn ref_delta<T>(&mut self, key: StoragePtr<T>, delta: i64) -> Result<(), E>;
  fn flush_ref_deltas(&mut self) -> Result<(), E>;

  fn cache_get(&mut self, key: RcKey) -> Option<StorageEntryPtr>;
  fn store_and_cache(&mut self, value: &StorageVal, key: RcKey) -> Result<StorageEntryPtr, E>;
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

  fn get_head(&mut self, ptr: StorageHeadPtr) -> Result<Option<StorageVal>, E> {
    let key = match self.read(ptr)? {
      Some(key) => key,
      None => return Ok(None),
    };

    let value = self.read(key)?.map(StorageVal::from_entry);

    Ok(value)
  }

  fn set_head(&mut self, ptr: StorageHeadPtr, value: Option<&StorageVal>) -> Result<(), E> {
    if let Some(value) = value {
      let key = self.store_with_replacements(value)?;
      self.ref_delta(key, 1)?;

      if let Some(old_key) = self.read(ptr)? {
        self.ref_delta(old_key, -1)?;
      }

      self.write(ptr, Some(&key))
    } else {
      if let Some(old_key) = self.read(ptr)? {
        self.ref_delta(old_key, -1)?;
      }

      self.write(ptr, None)
    }
  }

  fn store_with_replacements(&mut self, value: &StorageVal) -> Result<StorageEntryPtr, E> {
    if let Some(key) = value.maybe_replace_store(self)? {
      return Ok(key);
    }

    self.store(value)
  }

  fn store(&mut self, value: &StorageVal) -> Result<StorageEntryPtr, E> {
    let key = StoragePtr::random(&mut thread_rng());
    let entry = value.to_entry();
    self.write(key, Some(&entry))?;
    self.ref_delta(key, -1)?; // Cancel out the assumed single reference

    for subkey in &entry.refs {
      self.ref_delta(*subkey, 1)?;
    }

    Ok(key)
  }

  fn ref_delta<T>(&mut self, key: StoragePtr<T>, delta: i64) -> Result<(), E> {
    let ref_delta = self.ref_deltas().entry(key.data).or_insert(0);
    *ref_delta += delta;

    Ok(())
  }

  fn flush_ref_deltas(&mut self) -> Result<(), E> {
    while !self.ref_deltas().is_empty() {
      let ref_deltas = take(self.ref_deltas());

      for (key, delta) in ref_deltas.iter() {
        let delta = *delta;

        if delta <= 0 {
          continue;
        }

        let ptr = StorageEntryPtr::from_data(*key);

        let mut entry = match self.read(ptr)? {
          Some(entry) => entry,
          None => panic!("Key does not exist"),
        };

        entry.ref_count += delta as u64;

        self.write(ptr, Some(&entry))?;
      }

      for (key, delta) in ref_deltas.iter() {
        let delta = *delta;

        if delta >= 0 {
          continue;
        }

        let ptr = StorageEntryPtr::from_data(*key);

        let decrement = (-delta) as u64;

        let mut entry = match self.read(ptr)? {
          Some(entry) => entry,
          None => panic!("Key does not exist"),
        };

        if entry.ref_count == decrement {
          self.write(ptr, None)?;

          for subkey in entry.refs.iter() {
            self.ref_delta(*subkey, -1)?;
          }

          continue;
        }

        entry.ref_count -= decrement;

        self.write(ptr, Some(&entry))?;
      }
    }

    Ok(())
  }

  fn cache_get(&mut self, key: RcKey) -> Option<StorageEntryPtr> {
    self.cache().get(&key).cloned()
  }

  fn store_and_cache(&mut self, value: &StorageVal, key: RcKey) -> Result<StorageEntryPtr, E> {
    let ptr = self.store(value)?;

    let pre_existing = self.cache().insert(key, ptr);
    assert!(pre_existing.is_none());

    Ok(ptr)
  }
}
