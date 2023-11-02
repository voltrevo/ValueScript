use std::{cell::RefCell, collections::HashMap, mem::take, rc::Weak};

use rand::thread_rng;
use serde::{Deserialize, Serialize};

use crate::{
  GenericError, RcKey, StorageAutoPtr, StorageBackend, StorageEntity, StorageEntryPtr,
  StorageHeadPtr, StoragePtr,
};

pub trait StorageReader<'a, SB: StorageBackend>: Sized {
  fn read_bytes<T>(&self, ptr: StoragePtr<T>) -> Result<Option<Vec<u8>>, GenericError>;

  fn get_backend(&self) -> Weak<RefCell<SB>>;

  fn get_auto_ptr<SE: StorageEntity<SB>>(
    &mut self,
    ptr: StorageEntryPtr,
  ) -> StorageAutoPtr<SB, SE> {
    StorageAutoPtr {
      _marker: std::marker::PhantomData,
      sb: self.get_backend(),
      ptr,
    }
  }

  fn read<T: for<'de> Deserialize<'de>>(
    &mut self,
    ptr: StoragePtr<T>,
  ) -> Result<Option<T>, GenericError> {
    let data = match self.read_bytes(ptr)? {
      Some(data) => data,
      None => return Ok(None),
    };

    Ok(bincode::deserialize(&data).map(Some)?)
  }

  fn read_or_err<T: for<'de> Deserialize<'de>>(
    &mut self,
    ptr: StoragePtr<T>,
  ) -> Result<T, GenericError> {
    match self.read(ptr)? {
      Some(data) => Ok(data),
      None => Err("Ptr not found".into()),
    }
  }

  fn get_head<SE: StorageEntity<SB>>(
    &mut self,
    head_ptr: StorageHeadPtr,
  ) -> Result<Option<SE>, GenericError> {
    let entry_ptr = match self.read(head_ptr)? {
      Some(entry_ptr) => entry_ptr,
      None => return Ok(None),
    };

    let entry = match self.read(entry_ptr)? {
      Some(entry) => entry,
      None => return Ok(None),
    };

    SE::from_storage_entry(self, entry).map(Some)
  }
}

pub trait StorageTxMut<'a, SB: StorageBackend>: StorageReader<'a, SB> + Sized {
  fn ref_deltas(&mut self) -> &mut HashMap<(u64, u64, u64), i64>;
  fn cache(&mut self) -> &mut HashMap<RcKey, StorageEntryPtr>;
  fn write_bytes<T>(
    &mut self,
    ptr: StoragePtr<T>,
    data: Option<Vec<u8>>,
  ) -> Result<(), GenericError>;

  fn read_or_err<T: for<'de> Deserialize<'de>>(
    &mut self,
    ptr: StoragePtr<T>,
  ) -> Result<T, GenericError> {
    match self.read(ptr)? {
      Some(data) => Ok(data),
      None => Err("Ptr not found".into()),
    }
  }

  fn write<T: Serialize>(
    &mut self,
    ptr: StoragePtr<T>,
    data: Option<&T>,
  ) -> Result<(), GenericError> {
    let bytes = match data {
      Some(data) => Some(bincode::serialize(&data)?),
      None => None,
    };

    self.write_bytes(ptr, bytes)
  }

  fn get_head<SE: StorageEntity<SB>>(
    &mut self,
    head_ptr: StorageHeadPtr,
  ) -> Result<Option<SE>, GenericError> {
    let entry_ptr = match self.read(head_ptr)? {
      Some(entry_ptr) => entry_ptr,
      None => return Ok(None),
    };

    let entry = match self.read(entry_ptr)? {
      Some(entry) => entry,
      None => return Ok(None),
    };

    SE::from_storage_entry(self, entry).map(Some)
  }

  fn set_head<SE: StorageEntity<SB>>(
    &mut self,
    head_ptr: StorageHeadPtr,
    value: &SE,
  ) -> Result<(), GenericError> {
    let entry_ptr = self.store(value)?;
    self.ref_delta(entry_ptr, 1)?;

    if let Some(old_entry_ptr) = self.read(head_ptr)? {
      self.ref_delta(old_entry_ptr, -1)?;
    }

    self.write(head_ptr, Some(&entry_ptr))
  }

  fn remove_head(&mut self, head_ptr: StorageHeadPtr) -> Result<(), GenericError> {
    if let Some(old_entry_ptr) = self.read(head_ptr)? {
      self.ref_delta(old_entry_ptr, -1)?;
    }

    self.write(head_ptr, None)
  }

  fn store<SE: StorageEntity<SB>>(&mut self, value: &SE) -> Result<StorageEntryPtr, GenericError> {
    let ptr = StoragePtr::random(&mut thread_rng());
    let entry = value.to_storage_entry(self)?;
    self.write(ptr, Some(&entry))?;
    self.ref_delta(ptr, -1)?; // Cancel out the assumed single reference

    for subptr in &entry.refs {
      self.ref_delta(*subptr, 1)?;
    }

    Ok(ptr)
  }

  fn ref_delta<T>(&mut self, ptr: StoragePtr<T>, delta: i64) -> Result<(), GenericError> {
    let ref_delta = self.ref_deltas().entry(ptr.data).or_insert(0);
    *ref_delta += delta;

    Ok(())
  }

  fn flush_ref_deltas(&mut self) -> Result<(), GenericError> {
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

  fn store_and_cache<SE: StorageEntity<SB>>(
    &mut self,
    value: &SE,
    key: RcKey,
  ) -> Result<StorageEntryPtr, GenericError> {
    let ptr = self.store(value)?;

    let pre_existing = self.cache().insert(key, ptr);
    assert!(pre_existing.is_none());

    Ok(ptr)
  }
}
