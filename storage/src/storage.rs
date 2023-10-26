use std::{fmt::Debug as DebugTrait, rc::Rc};

use rand::thread_rng;
use serde::{Deserialize, Serialize};

use crate::serde_rc::{deserialize_rc, serialize_rc};
use crate::storage_ptr::{tmp_at_ptr, tmp_count_ptr, StorageEntryPtr, StorageHeadPtr, StoragePtr};

pub struct Storage<SB: StorageBackend> {
  sb: SB,
}

pub trait StorageBackendHandle<'a, E> {
  fn read<T>(&self, key: StoragePtr<T>) -> Result<Option<Vec<u8>>, E>;
  fn write<T>(&mut self, key: StoragePtr<T>, data: Option<Vec<u8>>) -> Result<(), E>;
}

pub trait StorageOps<E> {
  fn read_t<T: for<'de> Deserialize<'de>>(&mut self, key: StoragePtr<T>) -> Result<Option<T>, E>;
  fn write_t<T: Serialize>(&mut self, key: StoragePtr<T>, data: Option<&T>) -> Result<(), E>;

  fn inc_ref(&mut self, key: StorageEntryPtr) -> Result<(), E>;
  fn dec_ref(&mut self, key: StorageEntryPtr) -> Result<(), E>;

  fn get_head(&mut self, ptr: StorageHeadPtr) -> Result<Option<StorageVal>, E>;
  fn set_head(&mut self, ptr: StorageHeadPtr, value: Option<&StorageVal>) -> Result<(), E>;
}

impl<'a, Handle, E> StorageOps<E> for Handle
where
  Handle: StorageBackendHandle<'a, E>,
{
  fn read_t<T: for<'de> Deserialize<'de>>(&mut self, key: StoragePtr<T>) -> Result<Option<T>, E> {
    let data = match self.read(key)? {
      Some(data) => data,
      None => return Ok(None),
    };

    Ok(Some(bincode::deserialize(&data).unwrap()))
  }

  fn write_t<T: Serialize>(&mut self, key: StoragePtr<T>, data: Option<&T>) -> Result<(), E> {
    self.write(key, data.map(|data| bincode::serialize(&data).unwrap()))
  }

  fn inc_ref(&mut self, key: StorageEntryPtr) -> Result<(), E> {
    let mut entry = match self.read_t(key)? {
      Some(entry) => entry,
      None => panic!("Key does not exist"),
    };

    entry.ref_count += 1;

    self.write_t(key, Some(&entry))
  }

  fn dec_ref(&mut self, key: StorageEntryPtr) -> Result<(), E> {
    let mut entry = match self.read_t(key)? {
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
      self.write_t(key, Some(&entry))
    }
  }

  fn get_head(&mut self, ptr: StorageHeadPtr) -> Result<Option<StorageVal>, E> {
    let key = match self.read_t(ptr)? {
      Some(key) => key,
      None => return Ok(None),
    };

    let data = match self.read(key)? {
      Some(data) => data,
      None => panic!("Head points to non-existent key"),
    };

    Ok(Some(
      bincode::deserialize::<StorageEntry>(&data)
        .unwrap()
        .deserialize(),
    ))
  }

  fn set_head(&mut self, ptr: StorageHeadPtr, value: Option<&StorageVal>) -> Result<(), E> {
    if let Some(value) = value {
      let key = StoragePtr::random(&mut thread_rng());
      self.write(key, Some(bincode::serialize(&value.serialize()).unwrap()))?;

      {
        // TODO: Performance: Identify overlapping keys and cancel out the inc+dec

        for subkey in value.refs.iter() {
          self.inc_ref(*subkey)?;
        }

        if let Some(old_key) = self.read_t(ptr)? {
          self.dec_ref(old_key)?;
        }
      }

      self.write_t(ptr, Some(&key))
    } else {
      if let Some(old_key) = self.read_t(ptr)? {
        self.dec_ref(old_key)?;
      }

      self.write_t(ptr, None)
    }
  }
}

pub trait StorageBackend {
  type Error<E: DebugTrait>: DebugTrait;
  type InTransactionError<E>;
  type Handle<'a, E>: StorageBackendHandle<'a, Self::InTransactionError<E>>;

  fn transaction<F, T, E: DebugTrait>(&mut self, f: F) -> Result<T, Self::Error<E>>
  where
    F: Fn(&mut Self::Handle<'_, E>) -> Result<T, Self::InTransactionError<E>>;
}

impl<SB: StorageBackend> Storage<SB> {
  pub fn new(sb: SB) -> Self {
    Self { sb }
  }

  pub fn get_head(&mut self, ptr: StorageHeadPtr) -> Result<Option<StorageVal>, SB::Error<()>> {
    self.sb.transaction(|sb| sb.get_head(ptr))
  }

  pub fn set_head(
    &mut self,
    ptr: StorageHeadPtr,
    value: Option<&StorageVal>,
  ) -> Result<(), SB::Error<()>> {
    self.sb.transaction(|sb| sb.set_head(ptr, value))
  }

  pub fn store_tmp(&mut self, value: &StorageVal) -> Result<StorageEntryPtr, SB::Error<()>> {
    self.sb.transaction(|sb| {
      let key = StoragePtr::random(&mut thread_rng());
      sb.write(key, Some(bincode::serialize(value).unwrap()))?;

      let tmp_count = sb.read_t(tmp_count_ptr())?.unwrap_or(0);

      sb.write(tmp_at_ptr(tmp_count), Some(key.to_bytes()))?;

      sb.write(
        tmp_count_ptr(),
        Some(bincode::serialize(&(tmp_count + 1)).unwrap()),
      )?;

      Ok(key)
    })
  }

  pub fn clear_tmp(&mut self) -> Result<(), SB::Error<()>> {
    self.sb.transaction(|sb| {
      let tmp_count = sb.read_t(tmp_count_ptr())?.unwrap_or(0);

      for i in 0..tmp_count {
        let tmp_key = tmp_at_ptr(i);

        let entry_ptr = sb
          .read_t(tmp_key)?
          .unwrap_or_else(|| panic!("Missing tmp key"));

        sb.dec_ref(entry_ptr)?;
      }

      sb.write(tmp_count_ptr(), None)?;

      Ok(())
    })
  }
}

#[derive(Serialize, Deserialize)]
pub struct StorageEntry {
  ref_count: u64,

  #[serde(serialize_with = "serialize_rc", deserialize_with = "deserialize_rc")]
  refs: Rc<Vec<StorageEntryPtr>>,

  data: Vec<u8>,
}

#[derive(Default, Serialize, Deserialize, PartialEq, Eq, Debug)]
pub enum StoragePoint {
  #[default]
  Void,
  Number(u64),
  Array(
    #[serde(serialize_with = "serialize_rc", deserialize_with = "deserialize_rc")]
    Rc<Vec<StoragePoint>>,
  ),
  Ref(u64),
}

#[derive(Serialize, Deserialize)]
pub struct StorageVal {
  pub point: StoragePoint,

  #[serde(serialize_with = "serialize_rc", deserialize_with = "deserialize_rc")]
  pub refs: Rc<Vec<StorageEntryPtr>>,
}

impl StorageEntry {
  pub fn deserialize(&self) -> StorageVal {
    StorageVal {
      point: bincode::deserialize(&self.data).unwrap(),
      refs: self.refs.clone(),
    }
  }
}

impl StorageVal {
  pub fn serialize(&self) -> StorageEntry {
    StorageEntry {
      ref_count: 1,
      refs: self.refs.clone(),
      data: bincode::serialize(&self.point).unwrap(),
    }
  }
}
