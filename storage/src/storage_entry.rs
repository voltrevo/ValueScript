use serde::{Deserialize, Serialize};

use crate::storage_ptr::StorageEntryPtr;

#[derive(Serialize, Deserialize)]
pub struct StorageEntry {
  pub(crate) ref_count: u64,
  pub(crate) refs: Vec<StorageEntryPtr>,
  pub(crate) data: Vec<u8>,
}
