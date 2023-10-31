use std::fmt::Debug as DebugTrait;

use crate::storage_tx::StorageTx;

pub trait StorageBackend {
  type Error<E: DebugTrait>: DebugTrait;
  type InTxError<E>;
  type Tx<'a, E>: StorageTx<'a, Self::InTxError<E>>;

  fn transaction<F, T, E: DebugTrait>(&mut self, f: F) -> Result<T, Self::Error<E>>
  where
    F: Fn(&mut Self::Tx<'_, E>) -> Result<T, Self::InTxError<E>>;

  fn is_empty(&self) -> bool;

  #[cfg(test)]
  fn len(&self) -> usize;
}
