use std::error::Error;
use std::rc::Rc;

use crate::rc_key::RcKey;
use crate::storage_backend::StorageError;
use crate::storage_entity::StorageEntity;
use crate::storage_entry::{StorageEntry, StorageEntryReader};
use crate::storage_ptr::StorageEntryPtr;
use crate::storage_tx::StorageTx;
use crate::{Storage, StorageBackend};

const NUMBER_TAG: u8 = 0;
const ARRAY_TAG: u8 = 1;
const PTR_TAG: u8 = 2;

#[derive(Debug, Clone)]
pub enum DemoVal {
  Number(u64),
  Array(Rc<Vec<DemoVal>>),
  Ptr(StorageEntryPtr),
}

impl DemoVal {
  pub(crate) fn numbers<SB: StorageBackend>(
    &self,
    storage: &mut Storage<SB>,
  ) -> Result<Vec<u64>, Box<dyn Error>> {
    storage.sb.transaction(|sb| self.numbers_impl(sb))
  }

  fn write_to_entry<'a, SB: StorageBackend, Tx: StorageTx<'a, SB>>(
    &self,
    tx: &mut Tx,
    entry: &mut StorageEntry,
  ) -> Result<(), StorageError<SB>> {
    match self {
      DemoVal::Number(n) => {
        entry.data.push(NUMBER_TAG);
        entry.data.extend(n.to_le_bytes());
      }
      DemoVal::Array(arr) => 'b: {
        let key = RcKey::from(arr.clone());

        if let Some(ptr) = tx.cache_get(key.clone()) {
          entry.data.push(PTR_TAG);
          entry.refs.push(ptr);
          break 'b;
        }

        let ptr = tx.store_and_cache(self, key)?;
        entry.data.push(PTR_TAG);
        entry.refs.push(ptr);
      }
      DemoVal::Ptr(ptr) => {
        entry.data.push(PTR_TAG);
        entry.refs.push(*ptr);
      }
    };

    Ok(())
  }

  fn read_from_entry<'a, SB: StorageBackend, Tx: StorageTx<'a, SB>>(
    _tx: &mut Tx,
    reader: &mut StorageEntryReader,
  ) -> Result<DemoVal, StorageError<SB>> {
    let tag = reader.read_u8().map_err(StorageError::from)?;

    Ok(match tag {
      NUMBER_TAG => {
        let n = reader.read_u64().map_err(StorageError::from)?;
        DemoVal::Number(n)
      }
      ARRAY_TAG => {
        let len = reader.read_u64().map_err(StorageError::from)?;
        let mut items = Vec::new();

        for _ in 0..len {
          items.push(DemoVal::read_from_entry(_tx, reader)?);
        }

        DemoVal::Array(Rc::new(items))
      }
      PTR_TAG => {
        let ptr = reader.read_ref().map_err(StorageError::from)?;
        DemoVal::Ptr(ptr)
      }
      _ => panic!("Invalid tag"),
    })
  }

  fn numbers_impl<'a, SB: StorageBackend, Tx: StorageTx<'a, SB>>(
    &self,
    tx: &mut Tx,
  ) -> Result<Vec<u64>, StorageError<SB>> {
    match &self {
      DemoVal::Number(n) => Ok(vec![*n]),
      DemoVal::Ptr(ptr) => {
        let entry = tx.read_or_err(*ptr)?;
        Self::from_storage_entry(tx, entry)?.numbers_impl(tx)
      }
      DemoVal::Array(arr) => {
        let mut numbers = Vec::new();

        for item in arr.iter() {
          numbers.extend(item.numbers_impl(tx)?);
        }

        Ok(numbers)
      }
    }
  }
}

impl<'a, SB: StorageBackend, Tx: StorageTx<'a, SB>> StorageEntity<'a, SB, Tx> for DemoVal {
  fn to_storage_entry(&self, tx: &mut Tx) -> Result<StorageEntry, StorageError<SB>> {
    let mut entry = StorageEntry {
      ref_count: 1,
      refs: Vec::new(),
      data: Vec::new(),
    };

    match self {
      DemoVal::Array(arr) => {
        entry.data.push(ARRAY_TAG);

        entry
          .data
          .extend_from_slice(&(arr.len() as u64).to_le_bytes());

        for item in arr.iter() {
          item.write_to_entry(tx, &mut entry)?;
        }
      }
      _ => {
        self.write_to_entry(tx, &mut entry)?;
      }
    };

    Ok(entry)
  }

  fn from_storage_entry(tx: &mut Tx, entry: StorageEntry) -> Result<Self, StorageError<SB>> {
    Self::read_from_entry(tx, &mut StorageEntryReader::new(&entry))
  }
}
