use std::error::Error;

use crate::storage_tx::StorageTx;

pub trait StorageBackend: Sized {
  type CustomError;
  type Tx<'a>: StorageTx<'a, Self>;

  fn transaction<F, T>(&mut self, f: F) -> Result<T, Box<dyn Error>>
  where
    F: Fn(&mut Self::Tx<'_>) -> Result<T, StorageError<Self>>;

  fn is_empty(&self) -> bool;

  #[cfg(test)]
  fn len(&self) -> usize;
}

pub enum StorageError<SB: StorageBackend> {
  CustomError(SB::CustomError),
  Error(Box<dyn Error>),
}

pub fn to_storage_error<SB: StorageBackend, E: Error + 'static>(e: Box<E>) -> StorageError<SB> {
  StorageError::Error(e)
}

pub fn box_to_storage_error<SB: StorageBackend, E: Error + 'static>(e: E) -> StorageError<SB> {
  StorageError::Error(Box::new(e))
}
