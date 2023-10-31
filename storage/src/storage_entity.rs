use crate::{storage_backend_handle::StorageBackendHandle, storage_entry::StorageEntry};

pub trait StorageEntity: Sized {
  fn to_storage_entry<'a, E, Tx: StorageBackendHandle<'a, E>>(
    &self,
    tx: &mut Tx,
  ) -> Result<StorageEntry, E>;

  fn from_storage_entry<'a, E, Tx: StorageBackendHandle<'a, E>>(
    tx: &mut Tx,
    entry: StorageEntry,
  ) -> Result<Self, E>;
}
