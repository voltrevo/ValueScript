use crate::{storage_backend_handle::StorageBackendHandle, storage_entry::StorageEntry};

pub trait StorageEntity<'a, E, Tx: StorageBackendHandle<'a, E>>: Sized {
  fn to_storage_entry(&self, tx: &mut Tx) -> Result<StorageEntry, E>;
  fn from_storage_entry(tx: &mut Tx, entry: StorageEntry) -> Result<Self, E>;
}
