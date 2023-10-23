use std::rc::Rc;

use rand::{rngs::ThreadRng, thread_rng, Rng};

use serde::{ser::SerializeSeq, Serialize, Serializer};

#[derive(serde::Serialize, serde::Deserialize, Hash, PartialEq, Eq, Clone, Copy, Debug)]
pub struct StorageKey(u64, u64, u64);

pub struct Storage<RS: RawStorage> {
  rng: ThreadRng,
  rs: RS,
}

pub trait RawStorage {
  fn read(&self, key: StorageKey) -> Option<Vec<u8>>;
  fn write(&mut self, key: StorageKey, data: Option<Vec<u8>>);
}

impl<RS: RawStorage> Storage<RS> {
  pub fn new(rs: RS) -> Self {
    Self {
      rng: thread_rng(),
      rs,
    }
  }

  pub fn random_key(&mut self) -> StorageKey {
    StorageKey(self.rng.gen(), self.rng.gen(), self.rng.gen())
  }

  pub fn write_number(&mut self, number: f64) -> StorageKey {
    let mut data = Vec::<u8>::new();
    data.extend_from_slice(&number.to_le_bytes());
    let key = self.random_key();
    self.rs.write(key, Some(data));
    key
  }

  pub fn read_number(&self, key: StorageKey) -> Option<f64> {
    let data = self.rs.read(key)?;
    let mut bytes = [0u8; 8];
    bytes.copy_from_slice(&data);
    Some(f64::from_le_bytes(bytes))
  }
}

#[derive(serde::Serialize, serde::Deserialize)]
struct StoredRc {
  ref_count: u64,
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
