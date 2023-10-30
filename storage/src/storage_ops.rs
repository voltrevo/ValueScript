use std::mem::take;

use rand::thread_rng;
use serde::{Deserialize, Serialize};

use crate::{
  rc_key::RcKey,
  storage_backend::StorageBackendHandle,
  storage_entity::StorageEntity,
  storage_ptr::{StorageEntryPtr, StorageHeadPtr, StoragePtr},
  storage_val::StorageVal,
};

pub trait StorageOps<E> {
  fn read<T: for<'de> Deserialize<'de>>(&mut self, ptr: StoragePtr<T>) -> Result<Option<T>, E>;
  fn write<T: Serialize>(&mut self, ptr: StoragePtr<T>, data: Option<&T>) -> Result<(), E>;

  fn get_head(&mut self, ptr: StorageHeadPtr) -> Result<Option<StorageVal>, E>;
  fn set_head(&mut self, ptr: StorageHeadPtr, value: Option<&StorageVal>) -> Result<(), E>;

  fn store(&mut self, value: &StorageVal) -> Result<StorageEntryPtr, E>;

  fn ref_delta<T>(&mut self, ptr: StoragePtr<T>, delta: i64) -> Result<(), E>;
  fn flush_ref_deltas(&mut self) -> Result<(), E>;

  fn cache_get(&mut self, key: RcKey) -> Option<StorageEntryPtr>;
  fn store_and_cache(&mut self, value: &StorageVal, key: RcKey) -> Result<StorageEntryPtr, E>;
}

impl<'a, Handle, E> StorageOps<E> for Handle
where
  Handle: StorageBackendHandle<'a, E>,
{
  fn read<T: for<'de> Deserialize<'de>>(&mut self, ptr: StoragePtr<T>) -> Result<Option<T>, E> {
    let data = match self.read_bytes(ptr)? {
      Some(data) => data,
      None => return Ok(None),
    };

    Ok(Some(bincode::deserialize(&data).unwrap()))
  }

  fn write<T: Serialize>(&mut self, ptr: StoragePtr<T>, data: Option<&T>) -> Result<(), E> {
    self.write_bytes(ptr, data.map(|data| bincode::serialize(&data).unwrap()))
  }

  fn get_head(&mut self, head_ptr: StorageHeadPtr) -> Result<Option<StorageVal>, E> {
    let entry_ptr = match self.read(head_ptr)? {
      Some(entry_ptr) => entry_ptr,
      None => return Ok(None),
    };

    let entry = match self.read(entry_ptr)? {
      Some(entry) => entry,
      None => return Ok(None),
    };

    StorageVal::from_storage_entry(self, entry).map(Some)
  }

  fn set_head(&mut self, head_ptr: StorageHeadPtr, value: Option<&StorageVal>) -> Result<(), E> {
    if let Some(value) = value {
      let entry_ptr = self.store(value)?;
      self.ref_delta(entry_ptr, 1)?;

      if let Some(old_entry_ptr) = self.read(head_ptr)? {
        self.ref_delta(old_entry_ptr, -1)?;
      }

      self.write(head_ptr, Some(&entry_ptr))
    } else {
      if let Some(old_entry_ptr) = self.read(head_ptr)? {
        self.ref_delta(old_entry_ptr, -1)?;
      }

      self.write(head_ptr, None)
    }
  }

  fn store(&mut self, value: &StorageVal) -> Result<StorageEntryPtr, E> {
    let ptr = StoragePtr::random(&mut thread_rng());
    let entry = value.to_storage_entry(self)?;
    self.write(ptr, Some(&entry))?;
    self.ref_delta(ptr, -1)?; // Cancel out the assumed single reference

    for subptr in &entry.refs {
      self.ref_delta(*subptr, 1)?;
    }

    Ok(ptr)
  }

  fn ref_delta<T>(&mut self, ptr: StoragePtr<T>, delta: i64) -> Result<(), E> {
    let ref_delta = self.ref_deltas().entry(ptr.data).or_insert(0);
    *ref_delta += delta;

    Ok(())
  }

  fn flush_ref_deltas(&mut self) -> Result<(), E> {
    while !self.ref_deltas().is_empty() {
      let ref_deltas = take(self.ref_deltas());

      for (ptr, delta) in ref_deltas.iter() {
        let delta = *delta;

        if delta <= 0 {
          continue;
        }

        let ptr = StorageEntryPtr::from_data(*ptr);

        let mut entry = match self.read(ptr)? {
          Some(entry) => entry,
          None => panic!("Ptr not found"),
        };

        entry.ref_count += delta as u64;

        self.write(ptr, Some(&entry))?;
      }

      for (ptr, delta) in ref_deltas.iter() {
        let delta = *delta;

        if delta >= 0 {
          continue;
        }

        let ptr = StorageEntryPtr::from_data(*ptr);

        let decrement = (-delta) as u64;

        let mut entry = match self.read(ptr)? {
          Some(entry) => entry,
          None => panic!("Ptr not found"),
        };

        if entry.ref_count == decrement {
          self.write(ptr, None)?;

          for subptr in entry.refs.iter() {
            self.ref_delta(*subptr, -1)?;
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
