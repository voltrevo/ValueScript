use std::rc::Rc;

use crate::rc_key::RcKey;
use crate::storage_entity::StorageEntity;
use crate::storage_entry::{StorageEntry, StorageEntryReader};
use crate::storage_ops::StorageOps;
use crate::storage_ptr::StorageEntryPtr;
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
  ) -> Result<Vec<u64>, SB::Error<()>> {
    storage.sb.transaction(|sb| self.numbers_impl(sb))
  }

  fn write_to_entry<E, SO: StorageOps<E>>(
    &self,
    tx: &mut SO,
    entry: &mut StorageEntry,
  ) -> Result<(), E> {
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

  fn read_from_entry<E, SO: StorageOps<E>>(
    _tx: &mut SO,
    reader: &mut StorageEntryReader,
  ) -> Result<DemoVal, E> {
    let tag = reader.read_u8().unwrap();

    Ok(match tag {
      NUMBER_TAG => {
        let n = reader.read_u64().unwrap();
        DemoVal::Number(n)
      }
      ARRAY_TAG => {
        let len = reader.read_u64().unwrap();
        let mut items = Vec::new();

        for _ in 0..len {
          items.push(DemoVal::read_from_entry(_tx, reader)?);
        }

        DemoVal::Array(Rc::new(items))
      }
      PTR_TAG => {
        let ptr = reader.read_ref().unwrap();
        DemoVal::Ptr(ptr)
      }
      _ => panic!("Invalid tag"),
    })
  }

  fn numbers_impl<E, SO: StorageOps<E>>(&self, tx: &mut SO) -> Result<Vec<u64>, E> {
    match &self {
      DemoVal::Number(n) => Ok(vec![*n]),
      DemoVal::Ptr(ptr) => {
        let entry = tx.read(*ptr)?.unwrap();
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

impl StorageEntity for DemoVal {
  fn to_storage_entry<E, Tx: StorageOps<E>>(&self, tx: &mut Tx) -> Result<StorageEntry, E> {
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

  fn from_storage_entry<E, Tx: StorageOps<E>>(tx: &mut Tx, entry: StorageEntry) -> Result<Self, E> {
    let mut reader = StorageEntryReader::new(&entry);

    Self::read_from_entry(tx, &mut reader)
  }
}
