use crate::{storage_entry::StorageEntry, storage_tx::StorageTx};

pub trait StorageEntity<'a, E, Tx: StorageTx<'a, E>>: Sized {
  fn to_storage_entry(&self, tx: &mut Tx) -> Result<StorageEntry, E>;
  fn from_storage_entry(tx: &mut Tx, entry: StorageEntry) -> Result<Self, E>;
}
