use std::{cell::RefCell, error::Error, rc::Weak};

use crate::{
  storage_io::{StorageReader, StorageTxMut},
  GenericError, StoragePtr,
};

pub trait StorageBackend: Sized {
  type CustomError;
  type Tx<'a>: StorageReader<Self>;
  type TxMut<'a>: StorageTxMut<Self>;

  fn read_bytes<T>(&self, ptr: StoragePtr<T>) -> Result<Option<Vec<u8>>, GenericError>;

  fn transaction<F, T>(&self, self_weak: Weak<RefCell<Self>>, f: F) -> Result<T, Box<dyn Error>>
  where
    F: Fn(&mut Self::Tx<'_>) -> Result<T, GenericError>;

  fn transaction_mut<F, T>(
    &mut self,
    self_weak: Weak<RefCell<Self>>,
    f: F,
  ) -> Result<T, Box<dyn Error>>
  where
    F: Fn(&mut Self::TxMut<'_>) -> Result<T, GenericError>;

  fn is_empty(&self) -> bool;

  #[cfg(test)]
  fn len(&self) -> usize;
}
