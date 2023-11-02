use crate::{
  storage_backend::StorageError,
  storage_entry::StorageEntry,
  storage_tx::{StorageReader, StorageTxMut},
  StorageBackend,
};

pub trait StorageEntity<SB: StorageBackend>: Sized {
  fn to_storage_entry<'a, Tx: StorageTxMut<'a, SB>>(
    &self,
    tx: &mut Tx,
  ) -> Result<StorageEntry, StorageError<SB>>;

  fn from_storage_entry<'a, Tx: StorageReader<'a, SB>>(
    tx: &mut Tx,
    entry: StorageEntry,
  ) -> Result<Self, StorageError<SB>>;
}
