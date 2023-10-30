use crate::{storage_entry::StorageEntry, storage_ops::StorageOps};

pub trait Storable {
  fn to_storage_entry<E, Tx: StorageOps<E>>(&self, tx: &mut Tx) -> StorageEntry;
  fn from_storage_entry<E, Tx: StorageOps<E>>(tx: &mut Tx, entry: StorageEntry) -> Self;
}
