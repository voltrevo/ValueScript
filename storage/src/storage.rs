use std::{fmt::Debug as DebugTrait, rc::Rc};

use rand::thread_rng;
use serde::{Deserialize, Serialize};

use crate::serde_rc::{deserialize_rc, serialize_rc};
use crate::storage_ptr::{tmp_at_ptr, tmp_count_ptr, StorageEntryPtr, StorageHeadPtr, StoragePtr};

pub struct Storage<SB: StorageBackend> {
  sb: SB,
}

pub trait StorageBackendHandle<'a, E> {
  fn read_bytes<T>(&self, key: StoragePtr<T>) -> Result<Option<Vec<u8>>, E>;
  fn write_bytes<T>(&mut self, key: StoragePtr<T>, data: Option<Vec<u8>>) -> Result<(), E>;
}

pub trait StorageOps<E> {
  fn read<T: for<'de> Deserialize<'de>>(&mut self, key: StoragePtr<T>) -> Result<Option<T>, E>;
  fn write<T: Serialize>(&mut self, key: StoragePtr<T>, data: Option<&T>) -> Result<(), E>;

  fn inc_ref(&mut self, key: StorageEntryPtr) -> Result<(), E>;
  fn dec_ref(&mut self, key: StorageEntryPtr) -> Result<(), E>;

  fn get_head(&mut self, ptr: StorageHeadPtr) -> Result<Option<StorageVal>, E>;
  fn set_head(&mut self, ptr: StorageHeadPtr, value: Option<&StorageVal>) -> Result<(), E>;
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

      self.write_bytes(key, None)
    } else {
      self.write(key, Some(&entry))
    }
  }

  fn get_head(&mut self, ptr: StorageHeadPtr) -> Result<Option<StorageVal>, E> {
    let key = match self.read(ptr)? {
      Some(key) => key,
      None => return Ok(None),
    };

    let data = match self.read_bytes(key)? {
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
      self.write_bytes(key, Some(bincode::serialize(&value.serialize()).unwrap()))?;

      {
        // TODO: Performance: Identify overlapping keys and cancel out the inc+dec

        for subkey in value.refs.iter() {
          self.inc_ref(*subkey)?;
        }

        if let Some(old_key) = self.read(ptr)? {
          self.dec_ref(old_key)?;
        }
      }

      self.write(ptr, Some(&key))
    } else {
      if let Some(old_key) = self.read(ptr)? {
        self.dec_ref(old_key)?;
      }

      self.write(ptr, None)
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
      let tmp_count = sb.read(tmp_count_ptr())?.unwrap_or(0);
      let tmp_ptr = tmp_at_ptr(tmp_count);
      sb.set_head(tmp_ptr, Some(value))?;

      sb.write_bytes(
        tmp_count_ptr(),
        Some(bincode::serialize(&(tmp_count + 1)).unwrap()),
      )?;

      let key = sb
        .read(tmp_ptr)?
        .unwrap_or_else(|| panic!("Missing tmp key"));

      Ok(key)
    })
  }

  pub fn clear_tmp(&mut self) -> Result<(), SB::Error<()>> {
    self.sb.transaction(|sb| {
      let tmp_count = sb.read(tmp_count_ptr())?.unwrap_or(0);

      for i in 0..tmp_count {
        sb.set_head(tmp_at_ptr(i), None)?;
      }

      sb.write(tmp_count_ptr(), None)?;

      Ok(())
    })
  }

  pub(crate) fn get_ref_count(
    &mut self,
    key: StorageEntryPtr,
  ) -> Result<Option<u64>, SB::Error<()>> {
    self
      .sb
      .transaction(|sb| Ok(sb.read(key)?.map(|entry| entry.ref_count)))
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
