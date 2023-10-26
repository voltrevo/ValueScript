mod memory_backend;
mod storage;

mod serde_rc;
mod sled_backend;
mod storage_ptr;
mod tests;

pub use self::storage::{Storage, StorageBackend};
pub use memory_backend::MemoryBackend;
pub use sled_backend::SledBackend;
pub use storage_ptr::StoragePtr;
