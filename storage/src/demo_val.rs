use std::rc::Rc;

use crate::rc_key::RcKey;
use crate::storage_entity::StorageEntity;
use crate::storage_entry::{StorageEntry, StorageEntryReader};
use crate::storage_io::{StorageReader, StorageTxMut};
use crate::storage_ptr::StorageEntryPtr;
use crate::{GenericError, StorageBackend};

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
  fn write_to_entry<SB: StorageBackend, Tx: StorageTxMut<SB>>(
    &self,
    tx: &mut Tx,
    entry: &mut StorageEntry,
  ) -> Result<(), GenericError> {
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

  fn read_from_entry<SB: StorageBackend, Tx: StorageReader<SB>>(
    _tx: &Tx,
    reader: &mut StorageEntryReader,
  ) -> Result<DemoVal, GenericError> {
    let tag = reader.read_u8()?;

    Ok(match tag {
      NUMBER_TAG => {
        let n = reader.read_u64()?;
        DemoVal::Number(n)
      }
      ARRAY_TAG => {
        let len = reader.read_u64()?;
        let mut items = Vec::new();

        for _ in 0..len {
          items.push(DemoVal::read_from_entry(_tx, reader)?);
        }

        DemoVal::Array(Rc::new(items))
      }
      PTR_TAG => {
        let ptr = reader.read_ref()?;
        DemoVal::Ptr(ptr)
      }
      _ => panic!("Invalid tag"),
    })
  }

  pub fn numbers<SB: StorageBackend, Tx: StorageReader<SB>>(
    &self,
    tx: &mut Tx,
  ) -> Result<Vec<u64>, GenericError> {
    match &self {
      DemoVal::Number(n) => Ok(vec![*n]),
      DemoVal::Ptr(ptr) => {
        let entry = tx.read_or_err(*ptr)?;
        Self::from_storage_entry(tx, entry)?.numbers(tx)
      }
      DemoVal::Array(arr) => {
        let mut numbers = Vec::new();

        for item in arr.iter() {
          numbers.extend(item.numbers(tx)?);
        }

        Ok(numbers)
      }
    }
  }
}

impl<SB: StorageBackend> StorageEntity<SB> for DemoVal {
  fn to_storage_entry<'a, Tx: StorageTxMut<SB>>(
    &self,
    tx: &mut Tx,
  ) -> Result<StorageEntry, GenericError> {
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

  fn from_storage_entry<'a, Tx: StorageReader<SB>>(
    tx: &Tx,
    entry: StorageEntry,
  ) -> Result<Self, GenericError> {
    Self::read_from_entry(tx, &mut StorageEntryReader::new(&entry))
  }
}
