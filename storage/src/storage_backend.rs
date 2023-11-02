use std::{cell::RefCell, error::Error, rc::Weak};

use crate::storage_tx::{StorageTx, StorageTxMut};

pub trait StorageBackend: Sized {
  type CustomError;
  type Tx<'a>: StorageTx<'a, Self>;
  type TxMut<'a>: StorageTxMut<'a, Self>;

  fn transaction<F, T>(&self, self_weak: Weak<RefCell<Self>>, f: F) -> Result<T, Box<dyn Error>>
  where
    F: Fn(&mut Self::Tx<'_>) -> Result<T, StorageError<Self>>;

  fn transaction_mut<F, T>(
    &mut self,
    self_weak: Weak<RefCell<Self>>,
    f: F,
  ) -> Result<T, Box<dyn Error>>
  where
    F: Fn(&mut Self::TxMut<'_>) -> Result<T, StorageError<Self>>;

  fn is_empty(&self) -> bool;

  #[cfg(test)]
  fn len(&self) -> usize;
}

pub enum StorageError<SB: StorageBackend> {
  CustomError(SB::CustomError),
  Error(Box<dyn Error>),
}

impl<SB: StorageBackend> From<Box<dyn Error>> for StorageError<SB> {
  fn from(e: Box<dyn Error>) -> Self {
    StorageError::Error(e)
  }
}

impl<SB: StorageBackend> StorageError<SB> {
  pub fn from<E: Error + 'static>(e: E) -> Self {
    StorageError::Error(Box::new(e))
  }
}
