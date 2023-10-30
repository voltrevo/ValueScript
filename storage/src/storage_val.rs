use std::rc::Rc;

use crate::rc_key::RcKey;
use crate::storage_entity::StorageEntity;
use crate::storage_entry::{StorageEntry, StorageEntryReader};
use crate::storage_ops::StorageOps;
use crate::storage_ptr::StorageEntryPtr;

#[cfg(test)]
use crate::{Storage, StorageBackend};

#[derive(Default, Debug, Clone)]
pub enum StorageVal {
  #[default]
  Void,
  Number(u64),
  Ptr(StorageEntryPtr),
  Compound(Rc<StorageCompoundVal>),
}

#[derive(Debug)]
pub enum StorageCompoundVal {
  Array(StorageArray),
}

#[derive(Debug)]
pub struct StorageArray {
  pub items: Vec<StorageVal>,
}

impl StorageVal {
  #[cfg(test)]
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
      StorageVal::Void => {
        entry.data.push(0);
      }
      StorageVal::Number(n) => {
        entry.data.push(1);
        entry.data.extend(n.to_le_bytes());
      }
      StorageVal::Ptr(ptr) => {
        entry.data.push(2);
        entry.refs.push(*ptr);
      }
      StorageVal::Compound(compound) => 'b: {
        let key = RcKey::from(compound.clone());

        if let Some(ptr) = tx.cache_get(key.clone()) {
          entry.data.push(2);
          entry.refs.push(ptr);
          break 'b;
        }

        let ptr = tx.store_and_cache(self, key)?;
        entry.data.push(2);
        entry.refs.push(ptr);
      }
    };

    Ok(())
  }

  fn read_from_entry<E, SO: StorageOps<E>>(
    _tx: &mut SO,
    reader: &mut StorageEntryReader,
  ) -> Result<StorageVal, E> {
    let tag = reader.read_u8().unwrap();

    Ok(match tag {
      0 => StorageVal::Void,
      1 => {
        let n = reader.read_u64().unwrap();
        StorageVal::Number(n)
      }
      2 => {
        let ptr = reader.read_ref().unwrap();
        StorageVal::Ptr(ptr)
      }
      3 => {
        let len = reader.read_u64().unwrap();
        let mut items = Vec::new();

        for _ in 0..len {
          items.push(StorageVal::read_from_entry(_tx, reader)?);
        }

        let compound = Rc::new(StorageCompoundVal::Array(StorageArray { items }));

        StorageVal::Compound(compound)
      }
      _ => panic!("Invalid tag"),
    })
  }

  #[cfg(test)]
  fn numbers_impl<E, SO: StorageOps<E>>(&self, tx: &mut SO) -> Result<Vec<u64>, E> {
    match &self {
      StorageVal::Void => Ok(Vec::new()),
      StorageVal::Number(n) => Ok(vec![*n]),
      StorageVal::Ptr(ptr) => {
        let entry = tx.read(*ptr)?.unwrap();
        Self::from_storage_entry(tx, entry)?.numbers_impl(tx)
      }
      StorageVal::Compound(compound) => match &**compound {
        StorageCompoundVal::Array(arr) => {
          let mut numbers = Vec::new();

          for item in &arr.items {
            numbers.extend(item.numbers_impl(tx)?);
          }

          Ok(numbers)
        }
      },
    }
  }
}

impl StorageEntity for StorageVal {
  fn to_storage_entry<E, Tx: StorageOps<E>>(&self, tx: &mut Tx) -> Result<StorageEntry, E> {
    let mut entry = StorageEntry {
      ref_count: 1,
      refs: Vec::new(),
      data: Vec::new(),
    };

    match self {
      StorageVal::Compound(compound) => match &**compound {
        StorageCompoundVal::Array(arr) => {
          entry.data.push(3);

          entry
            .data
            .extend_from_slice(&(arr.items.len() as u64).to_le_bytes());

          for item in &arr.items {
            item.write_to_entry(tx, &mut entry)?;
          }
        }
      },
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
