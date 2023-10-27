use std::fmt::Debug as DebugTrait;

use crate::storage_ptr::StoragePtr;

pub trait StorageBackendHandle<'a, E> {
  fn read_bytes<T>(&self, key: StoragePtr<T>) -> Result<Option<Vec<u8>>, E>;
  fn write_bytes<T>(&mut self, key: StoragePtr<T>, data: Option<Vec<u8>>) -> Result<(), E>;
}

pub trait StorageBackend {
  type Error<E: DebugTrait>: DebugTrait;
  type InTransactionError<E>;
  type Handle<'a, E>: StorageBackendHandle<'a, Self::InTransactionError<E>>;

  fn transaction<F, T, E: DebugTrait>(&mut self, f: F) -> Result<T, Self::Error<E>>
  where
    F: Fn(&mut Self::Handle<'_, E>) -> Result<T, Self::InTransactionError<E>>;

  fn is_empty(&self) -> bool;
}
