use std::{collections::HashMap, mem::take};

use rand::thread_rng;
use serde::{Deserialize, Serialize};

use crate::{
  storage_backend::StorageBackendHandle,
  storage_ptr::{StorageEntryPtr, StorageHeadPtr, StoragePtr},
  storage_val::StorageVal,
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

  fn flush_ref_deltas(&mut self) -> Result<(), E>;
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
    if let Some(delta) = self.ref_deltas().get_mut(&key.data) {
      *delta += 1;
      return Ok(());
    }

    let mut entry = match self.read(key)? {
      Some(entry) => entry,
      None => panic!("Key does not exist"),
    };

    entry.ref_count += 1;

    self.write(key, Some(&entry))
  }

  fn dec_ref(&mut self, key: StorageEntryPtr) -> Result<(), E> {
    if let Some(delta) = self.ref_deltas().get_mut(&key.data) {
      *delta -= 1;
      return Ok(());
    }

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
      self.buf_ref_delta(key, 1)?;

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
    let mut cache = HashMap::<u64, StorageEntryPtr>::new();

    if let Some(key) = value
      .point
      .maybe_replace_store(self, &value.refs, &mut cache)?
    {
      return Ok(key);
    }

    self.store(value)
  }

  fn store(&mut self, value: &StorageVal) -> Result<StorageEntryPtr, E> {
    let key = StoragePtr::random(&mut thread_rng());
    self.write(key, Some(&value.to_entry()))?;
    self.buf_ref_delta(key, -1)?; // Cancel out the assumed single reference

    for subkey in value.refs.iter() {
      self.buf_ref_delta(*subkey, 1)?;
    }

    Ok(key)
  }

  fn flush_ref_deltas(&mut self) -> Result<(), E> {
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
        continue;
      }

      entry.ref_count -= decrement;

      self.write(ptr, Some(&entry))?;
    }

    Ok(())
  }
}
