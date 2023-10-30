mod memory_backend;
mod storage;

mod rc_key;
mod serde_rc;
mod sled_backend;
mod storable;
mod storage_backend;
mod storage_entry;
mod storage_ops;
mod storage_ptr;
mod storage_val;
mod tests;

pub use self::storage::Storage;
pub use self::storage_backend::StorageBackend;
pub use memory_backend::MemoryBackend;
pub use sled_backend::SledBackend;
pub use storage_ptr::{storage_head_ptr, StoragePtr};
