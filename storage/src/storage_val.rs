use std::rc::Rc;

use serde::{Deserialize, Serialize};

use crate::rc_key::RcKey;
use crate::serde_rc::{deserialize_rc, serialize_rc};
use crate::storage_entity::StorageEntity;
use crate::storage_entry::StorageEntry;
use crate::storage_ops::StorageOps;
use crate::storage_ptr::StorageEntryPtr;

#[cfg(test)]
use crate::{Storage, StorageBackend};

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub enum StorageVal {
  #[default]
  Void,
  Number(u64),
  Ptr(StorageEntryPtr),
  Ref(u64),
  Compound(
    #[serde(serialize_with = "serialize_rc", deserialize_with = "deserialize_rc")]
    Rc<StorageCompoundVal>,
  ),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum StorageCompoundVal {
  Array(StorageArray),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StorageArray {
  pub items: Vec<StorageVal>,

  // Skipping serialization because they're stored in the entry. When converting from an entry, we
  // copy (todo: move?) the refs from there.
  #[serde(skip)]
  pub refs: Vec<StorageEntryPtr>,
}

impl StorageVal {
  #[cfg(test)]
  pub(crate) fn numbers<SB: StorageBackend>(
    &self,
    storage: &mut Storage<SB>,
  ) -> Result<Vec<u64>, SB::Error<()>> {
    storage.sb.transaction(|sb| self.numbers_impl(sb))
  }

  pub fn maybe_replace_store<E, SO: StorageOps<E>>(
    &self,
    tx: &mut SO,
  ) -> Result<Option<StorageEntryPtr>, E> {
    if let Some(id) = self.cache_key() {
      if let Some(ptr) = tx.cache_get(id) {
        return Ok(Some(ptr));
      }
    }

    Ok(match &self {
      StorageVal::Void | StorageVal::Number(_) | StorageVal::Ptr(_) | StorageVal::Ref(_) => None,
      StorageVal::Compound(compound) => match &**compound {
        StorageCompoundVal::Array(arr) => 'b: {
          let mut replacements: Vec<(usize, StorageEntryPtr)> = Vec::new();

          for i in 0..arr.items.len() {
            if let Some(ptr) = arr.items[i].maybe_replace_store(tx)? {
              replacements.push((i, ptr));
            }
          }

          let key = RcKey::from(compound.clone());

          if replacements.is_empty() {
            break 'b Some(tx.store_and_cache(self, key)?);
          }

          let mut new_arr = Vec::<StorageVal>::new();
          let mut new_refs = arr.refs.clone();

          let mut replacements_iter = replacements.iter();
          let mut next_replacement = replacements_iter.next();

          for (i, item) in arr.items.iter().enumerate() {
            if let Some((j, entry_ptr)) = next_replacement {
              if *j == i {
                new_arr.push(StorageVal::Ref(new_refs.len() as u64));
                new_refs.push(*entry_ptr);
                next_replacement = replacements_iter.next();
                continue;
              }
            }

            new_arr.push(item.clone());
          }

          Some(tx.store_and_cache(
            &StorageVal::Compound(Rc::new(StorageCompoundVal::Array(StorageArray {
              items: new_arr,
              refs: new_refs,
            }))),
            key,
          )?)
        }
      },
    })
  }

  fn cache_key(&self) -> Option<RcKey> {
    match self {
      StorageVal::Void => None,
      StorageVal::Number(_) => None,
      StorageVal::Ptr(_) => None,
      StorageVal::Ref(_) => None,
      StorageVal::Compound(compound) => Some(RcKey::from(compound.clone())),
    }
  }

  #[cfg(test)]
  fn numbers_impl<E, SO: StorageOps<E>>(&self, tx: &mut SO) -> Result<Vec<u64>, E> {
    match &self {
      StorageVal::Void => Ok(Vec::new()),
      StorageVal::Number(n) => Ok(vec![*n]),
      StorageVal::Ptr(ptr) => {
        let entry = tx.read(*ptr)?.unwrap();
        Self::from_storage_entry(tx, entry).numbers_impl(tx)
      }
      StorageVal::Ref(_) => {
        panic!("Can't lookup ref (shouldn't hit this case)")
      }
      StorageVal::Compound(compound) => match &**compound {
        StorageCompoundVal::Array(arr) => {
          let mut numbers = Vec::new();

          for item in &arr.items {
            if let StorageVal::Ref(i) = item {
              let item = &StorageVal::Ptr(arr.refs[*i as usize]);
              numbers.extend(item.numbers_impl(tx)?);
            } else {
              numbers.extend(item.numbers_impl(tx)?);
            }
          }

          Ok(numbers)
        }
      },
    }
  }
}

impl StorageEntity for StorageVal {
  fn to_storage_entry<E, Tx: StorageOps<E>>(&self, _tx: &mut Tx) -> StorageEntry {
    StorageEntry {
      ref_count: 1,
      refs: match self {
        StorageVal::Void | StorageVal::Number(_) | StorageVal::Ptr(_) | StorageVal::Ref(_) => {
          vec![]
        }
        StorageVal::Compound(compound) => match &**compound {
          StorageCompoundVal::Array(arr) => arr.refs.clone(),
        },
      },
      data: bincode::serialize(self).unwrap(),
    }
  }

  fn from_storage_entry<E, Tx: StorageOps<E>>(_tx: &mut Tx, entry: StorageEntry) -> Self {
    let StorageEntry {
      ref_count: _,
      refs,
      data,
    } = entry;

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
