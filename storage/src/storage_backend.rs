use std::error::Error;

use crate::storage_tx::StorageTx;

pub trait StorageBackend: Sized {
  type InTxError;
  type Tx<'a>: StorageTx<'a, Self>;

  fn transaction<F, T>(&mut self, f: F) -> Result<T, Box<dyn Error>>
  where
    F: Fn(&mut Self::Tx<'_>) -> Result<T, Self::InTxError>;

  fn is_empty(&self) -> bool;

  #[cfg(test)]
  fn len(&self) -> usize;
}
