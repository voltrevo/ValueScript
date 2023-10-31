use crate::{
  storage_backend::StorageError, storage_entry::StorageEntry, storage_tx::StorageTx, StorageBackend,
};

pub trait StorageEntity<'a, SB: StorageBackend, Tx: StorageTx<'a, SB>>: Sized {
  fn to_storage_entry(&self, tx: &mut Tx) -> Result<StorageEntry, StorageError<SB>>;

  fn from_storage_entry(tx: &mut Tx, entry: StorageEntry) -> Result<Self, StorageError<SB>>;
}
