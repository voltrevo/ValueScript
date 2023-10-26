use std::{fmt::Debug as DebugTrait, rc::Rc};

use rand::thread_rng;
use serde::{Deserialize, Serialize};

use crate::serde_rc::{deserialize_rc, serialize_rc};
use crate::storage_key::StorageKey;

pub struct Storage<SB: StorageBackend> {
  sb: SB,
}

pub trait StorageBackendHandle<'a, E> {
  fn read(&self, key: StorageKey) -> Result<Option<Vec<u8>>, E>;
  fn write(&mut self, key: StorageKey, data: Option<Vec<u8>>) -> Result<(), E>;
}

pub trait StorageOps<E> {
  fn read_entry(&mut self, key: StorageKey) -> Result<Option<StorageEntry>, E>;
  fn write_entry(&mut self, key: StorageKey, data: Option<&StorageEntry>) -> Result<(), E>;

  fn inc_ref(&mut self, key: StorageKey) -> Result<(), E>;
  fn dec_ref(&mut self, key: StorageKey) -> Result<(), E>;

  fn get_head_key(&mut self, name: &[u8]) -> Result<Option<StorageKey>, E>;
  fn set_head_key(&mut self, name: &[u8], key: Option<&StorageKey>) -> Result<(), E>;

  fn get_head(&mut self, name: &[u8]) -> Result<Option<StorageVal>, E>;
  fn set_head(&mut self, name: &[u8], value: Option<&StorageVal>) -> Result<(), E>;
}

impl<'a, T, E> StorageOps<E> for T
where
  T: StorageBackendHandle<'a, E>,
{
  fn read_entry(&mut self, key: StorageKey) -> Result<Option<StorageEntry>, E> {
    let data = match self.read(key)? {
      Some(data) => data,
      None => return Ok(None),
    };

    Ok(Some(bincode::deserialize(&data).unwrap()))
  }

  fn write_entry(&mut self, key: StorageKey, data: Option<&StorageEntry>) -> Result<(), E> {
    self.write(key, data.map(|data| bincode::serialize(&data).unwrap()))
  }

  fn inc_ref(&mut self, key: StorageKey) -> Result<(), E> {
    let mut entry = match self.read_entry(key)? {
      Some(entry) => entry,
      None => panic!("Key does not exist"),
    };

    entry.ref_count += 1;

    self.write_entry(key, Some(&entry))
  }

  fn dec_ref(&mut self, key: StorageKey) -> Result<(), E> {
    let mut entry = match self.read_entry(key)? {
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
      self.write_entry(key, Some(&entry))
    }
  }

  fn get_head_key(&mut self, name: &[u8]) -> Result<Option<StorageKey>, E> {
    match self.read(StorageKey::from_bytes(name))? {
      Some(key_bytes) => Ok(Some(StorageKey::from_bytes(&key_bytes))),
      None => Ok(None),
    }
  }

  fn set_head_key(&mut self, name: &[u8], key: Option<&StorageKey>) -> Result<(), E> {
    self.write(StorageKey::from_bytes(name), key.map(|key| key.to_bytes()))
  }

  fn get_head(&mut self, name: &[u8]) -> Result<Option<StorageVal>, E> {
    let key = match self.get_head_key(name)? {
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

  fn set_head(&mut self, name: &[u8], value: Option<&StorageVal>) -> Result<(), E> {
    if let Some(value) = value {
      let key = StorageKey::random(&mut thread_rng());
      self.write(key, Some(bincode::serialize(&value.serialize()).unwrap()))?;

      {
        // TODO: Performance: Identify overlapping keys and cancel out the inc+dec

        for subkey in value.refs.iter() {
          self.inc_ref(*subkey)?;
        }

        if let Some(old_key) = self.get_head_key(name)? {
          self.dec_ref(old_key)?;
        }
      }

      self.set_head_key(name, Some(&key))
    } else {
      if let Some(old_key) = self.get_head_key(name)? {
        self.dec_ref(old_key)?;
      }

      self.set_head_key(name, None)
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

  pub fn get_head(&mut self, name: &[u8]) -> Result<Option<StorageVal>, SB::Error<()>> {
    self.sb.transaction(|sb| sb.get_head(name))
  }

  pub fn set_head(&mut self, name: &[u8], value: Option<&StorageVal>) -> Result<(), SB::Error<()>> {
    self.sb.transaction(|sb| sb.set_head(name, value))
  }
}

#[derive(Serialize, Deserialize)]
pub struct StorageEntry {
  ref_count: u64,

  #[serde(serialize_with = "serialize_rc", deserialize_with = "deserialize_rc")]
  refs: Rc<Vec<StorageKey>>,

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
  pub refs: Rc<Vec<StorageKey>>,
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
