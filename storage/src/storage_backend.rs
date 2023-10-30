use std::{collections::HashMap, fmt::Debug as DebugTrait};

use crate::{
  rc_key::RcKey,
  storage_ptr::{StorageEntryPtr, StoragePtr},
};

pub trait StorageBackendHandle<'a, E> {
  fn ref_deltas(&mut self) -> &mut HashMap<(u64, u64, u64), i64>;
  fn cache(&mut self) -> &mut HashMap<RcKey, StorageEntryPtr>;
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

  #[cfg(test)]
  fn len(&self) -> usize;
}
