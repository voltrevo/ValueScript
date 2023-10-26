use std::{fmt::Debug as DebugTrait, rc::Rc};

use rand::{rngs::ThreadRng, thread_rng};
use serde::{Deserialize, Serialize};

use crate::serde_rc::{deserialize_rc, serialize_rc};
use crate::storage_key::StorageKey;

pub struct Storage<SB: StorageBackend> {
  rng: ThreadRng,
  sb: SB,
}

pub trait StorageBackendHandle<'a, E> {
  fn read(&self, key: StorageKey) -> Result<Option<Vec<u8>>, E>;
  fn write(&mut self, key: StorageKey, data: Option<Vec<u8>>) -> Result<(), E>;
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
    Self {
      rng: thread_rng(),
      sb,
    }
  }

  pub fn read(&mut self, key: StorageKey) -> Result<Option<StorageEntry>, SB::Error<()>> {
    self.sb.transaction(|sb| {
      let data = match sb.read(key)? {
        Some(data) => data,
        None => return Ok(None),
      };

      Ok(Some(bincode::deserialize(&data).unwrap()))
    })
  }

  pub fn write(&mut self, key: StorageKey, data: &StorageEntry) -> Result<(), SB::Error<()>> {
    self.sb.transaction(|sb| {
      let data = bincode::serialize(&data).unwrap();
      sb.write(key, Some(data))
    })
  }

  pub fn set_head(&mut self, name: &[u8], value: Option<&StorageVal>) -> Result<(), SB::Error<()>> {
    let Self { sb, rng } = self;

    sb.transaction(|sb| {
      let mut r = rng.clone();

      let key = StorageKey::random(&mut r);

      sb.write(
        key,
        value.map(|value| bincode::serialize(&value.serialize()).unwrap()),
      )?;

      // TODO: Handle existing
      sb.write(StorageKey::from_bytes(name), Some(key.to_bytes()))
    })
  }

  pub fn get_head(&mut self, name: &[u8]) -> Result<Option<StorageVal>, SB::Error<()>> {
    self.sb.transaction(|sb| {
      let key = match sb.read(StorageKey::from_bytes(name))? {
        Some(key_bytes) => StorageKey::from_bytes(&key_bytes),
        None => return Ok(None),
      };

      let data = match sb.read(key)? {
        Some(data) => data,
        None => panic!("Head points to non-existent key"),
      };

      Ok(Some(
        bincode::deserialize::<StorageEntry>(&data)
          .unwrap()
          .deserialize(),
      ))
    })
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
