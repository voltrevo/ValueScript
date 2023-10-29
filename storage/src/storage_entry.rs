use std::rc::Rc;

use serde::{Deserialize, Serialize};

use crate::{
  storage_ptr::StorageEntryPtr,
  storage_val::{StorageCompoundVal, StorageVal},
};

#[derive(Serialize, Deserialize)]
pub struct StorageEntry {
  pub(crate) ref_count: u64,
  pub(crate) refs: Vec<StorageEntryPtr>,
  pub(crate) data: Vec<u8>,
}

impl StorageEntry {
  pub fn move_to_val(self) -> StorageVal {
    let Self {
      ref_count: _,
      refs,
      data,
    } = self;

    let mut val = bincode::deserialize::<StorageVal>(&data).unwrap();

    if let StorageVal::Compound(compound) = &mut val {
      match Rc::get_mut(compound).expect("Should be single ref") {
        StorageCompoundVal::Array(arr) => {
          arr.refs = refs;
        }
      }
    };

    val
  }
}
