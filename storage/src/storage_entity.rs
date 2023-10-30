use crate::{storage_entry::StorageEntry, storage_ops::StorageOps};

pub trait StorageEntity: Sized {
  fn to_storage_entry<E, Tx: StorageOps<E>>(&self, tx: &mut Tx) -> Result<StorageEntry, E>;
  fn from_storage_entry<E, Tx: StorageOps<E>>(tx: &mut Tx, entry: StorageEntry) -> Result<Self, E>;
}
