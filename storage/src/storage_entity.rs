use crate::{storage_entry::StorageEntry, storage_tx::StorageTx, StorageBackend};

pub trait StorageEntity<'a, SB: StorageBackend, Tx: StorageTx<'a, SB>>: Sized {
  fn to_storage_entry(&self, tx: &mut Tx) -> Result<StorageEntry, SB::InTxError>;
  fn from_storage_entry(tx: &mut Tx, entry: StorageEntry) -> Result<Self, SB::InTxError>;
}
