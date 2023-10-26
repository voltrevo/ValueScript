use std::{fmt::Debug as DebugTrait, rc::Rc};

use rand::{rngs::ThreadRng, thread_rng, Rng};

#[derive(serde::Serialize, serde::Deserialize, Hash, PartialEq, Eq, Clone, Copy, Debug)]
pub struct StorageKey(u64, u64, u64);

impl StorageKey {
  pub fn from_bytes(bytes: &[u8]) -> Self {
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
}

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

  pub fn random_key(&mut self) -> StorageKey {
    StorageKey(self.rng.gen(), self.rng.gen(), self.rng.gen())
  }

  pub fn write_number(&mut self, number: f64) -> Result<StorageKey, SB::Error<()>> {
    let mut data = Vec::<u8>::new();
    data.extend_from_slice(&number.to_le_bytes());
    let key = self.random_key();

    self
      .sb
      .transaction(|sb| sb.write(key, Some(data.clone())))?;

    Ok(key)
  }

  pub fn read_number(&mut self, key: StorageKey) -> Result<Option<f64>, SB::Error<()>> {
    self.sb.transaction(|sb| {
      let data = match sb.read(key)? {
        Some(data) => data,
        None => return Ok(None),
      };

      let mut bytes = [0u8; 8];
      bytes.copy_from_slice(&data);
      Ok(Some(f64::from_le_bytes(bytes)))
    })
  }

  pub fn read(&mut self, key: StorageKey) -> Result<Option<StoredRc>, SB::Error<()>> {
    self.sb.transaction(|sb| {
      let data = match sb.read(key)? {
        Some(data) => data,
        None => return Ok(None),
      };

      Ok(Some(bincode::deserialize(&data).unwrap()))
    })
  }

  pub fn write(&mut self, key: StorageKey, data: &StoredRc) -> Result<(), SB::Error<()>> {
    self.sb.transaction(|sb| {
      let data = bincode::serialize(&data).unwrap();
      sb.write(key, Some(data))
    })
  }

  pub fn head(&mut self) -> Result<Option<StorageKey>, SB::Error<()>> {
    self.sb.transaction(|sb| {
      let data = match sb.read(StorageKey(0, 0, 0))? {
        Some(data) => data,
        None => return Ok(None),
      };

      Ok(Some(bincode::deserialize(&data).unwrap()))
    })
  }

  pub fn set_head(&mut self, key: StorageKey) -> Result<(), SB::Error<()>> {
    self.sb.transaction(|sb| {
      // TODO: read old head, deal with ref counts

      let data = bincode::serialize(&key).unwrap();
      sb.write(StorageKey(0, 0, 0), Some(data))
    })
  }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct StoredRc {
  count: u64,
  refs: Vec<StorageKey>,
  data: Vec<u8>,
}

#[derive(Default)]
enum StorageVal {
  #[default]
  Void,
  Number(f64),
  Array(Rc<Vec<StorageVal>>),
  Ref(u64),
}

struct StorageValWithRefs {
  val: StorageVal,
  refs: Rc<Vec<StorageKey>>,
}
