use crate::{
  storage_entry::StorageEntry,
  storage_tx::{StorageReader, StorageTxMut},
  GenericError, StorageBackend,
};

pub trait StorageEntity<SB: StorageBackend>: Sized {
  fn to_storage_entry<Tx: StorageTxMut<SB>>(
    &self,
    tx: &mut Tx,
  ) -> Result<StorageEntry, GenericError>;

  fn from_storage_entry<Tx: StorageReader<SB>>(
    tx: &mut Tx,
    entry: StorageEntry,
  ) -> Result<Self, GenericError>;
}
