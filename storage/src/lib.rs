mod memory_backend;
mod storage;

#[cfg(test)]
mod demo_val;

mod rc_key;
mod sled_backend;
mod storage_backend;
mod storage_entity;
mod storage_entry;
mod storage_ptr;
mod storage_tx;
mod tests;

pub use self::storage::Storage;
pub use self::storage_backend::StorageBackend;
pub use self::storage_tx::StorageTx;
pub use memory_backend::MemoryBackend;
pub use rc_key::RcKey;
pub use sled_backend::SledBackend;
pub use storage_entity::StorageEntity;
pub use storage_entry::{StorageEntry, StorageEntryReader, StorageEntryWriter};
pub use storage_ptr::{storage_head_ptr, StorageEntryPtr, StorageHeadPtr, StoragePtr};
