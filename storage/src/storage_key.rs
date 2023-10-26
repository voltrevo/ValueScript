use rand::{rngs::ThreadRng, Rng};

#[derive(serde::Serialize, serde::Deserialize, Hash, PartialEq, Eq, Clone, Copy, Debug)]
pub struct StorageKey(pub u64, pub u64, pub u64);

impl StorageKey {
  pub fn from_bytes(bytes: &[u8]) -> Self {
    if bytes.len() > 24 {
      panic!("Too many bytes");
    }

    let mut key = [0u8; 24];
    key[..bytes.len()].copy_from_slice(bytes);
    bincode::deserialize(&key).unwrap()
  }

  pub fn tmp_count() -> Self {
    let mut key = Self::from_bytes(b"tmp");
    key.2 = 1;

    key
  }

  pub fn tmp_at(i: u64) -> Self {
    let mut key = Self::tmp_count();
    key.2 = i + 2;

    key
  }

  pub fn to_bytes(&self) -> Vec<u8> {
    bincode::serialize(self).unwrap()
  }

  pub fn random(rng: &mut ThreadRng) -> Self {
    Self(rng.gen(), rng.gen(), rng.gen())
  }
}
