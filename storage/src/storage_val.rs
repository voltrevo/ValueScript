use std::collections::HashMap;
use std::rc::Rc;

use serde::{Deserialize, Serialize};

use crate::serde_rc::{deserialize_rc, serialize_rc};
use crate::storage_ops::StorageOps;
use crate::storage_ptr::StorageEntryPtr;

#[cfg(test)]
use crate::{Storage, StorageBackend};

#[derive(Serialize, Deserialize, Debug)]
pub struct StorageVal {
  pub point: StoragePoint,

  #[serde(serialize_with = "serialize_rc", deserialize_with = "deserialize_rc")]
  pub refs: Rc<Vec<StorageEntryPtr>>,
}

impl StorageVal {
  pub fn to_entry(&self) -> StorageEntry {
    StorageEntry {
      ref_count: 1,
      refs: self.refs.clone(),
      data: bincode::serialize(&self.point).unwrap(),
    }
  }

  #[cfg(test)]
  pub(crate) fn numbers<SB: StorageBackend>(
    &self,
    storage: &mut Storage<SB>,
  ) -> Result<Vec<u64>, SB::Error<()>> {
    storage
      .sb
      .transaction(|sb| self.point.numbers(sb, &self.refs))
  }
}

#[derive(Default, Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
pub enum StoragePoint {
  #[default]
  Void,
  Number(u64),
  Array(
    #[serde(serialize_with = "serialize_rc", deserialize_with = "deserialize_rc")]
    Rc<Vec<StoragePoint>>,
  ),
  Ref(u64),
}

impl StoragePoint {
  pub fn maybe_replace_store<E, SO: StorageOps<E>>(
    &self,
    tx: &mut SO,
    refs: &Rc<Vec<StorageEntryPtr>>,
    cache: &mut HashMap<u64, StorageEntryPtr>,
  ) -> Result<Option<StorageEntryPtr>, E> {
    if let Some(id) = self.cache_id() {
      if let Some(key) = cache.get(&id) {
        return Ok(Some(*key));
      }
    }

    Ok(match &self {
      StoragePoint::Void => None,
      StoragePoint::Number(_) => None,
      StoragePoint::Array(arr) => 'b: {
        let mut replacements: Vec<(usize, StorageEntryPtr)> = Vec::new();

        for i in 0..arr.len() {
          if let Some(key) = arr[i].maybe_replace_store(tx, refs, cache)? {
            replacements.push((i, key));
          }
        }

        if replacements.is_empty() {
          break 'b Some(cache_and_store(
            tx,
            &StorageVal {
              point: StoragePoint::Array(arr.clone()),
              refs: refs.clone(),
            },
            cache,
            cache_id(arr),
          )?);
        }

        let mut new_arr = Vec::<StoragePoint>::new();
        let mut new_refs = (**refs).clone();

        let mut replacements_iter = replacements.iter();
        let mut next_replacement = replacements_iter.next();

        for (i, point) in arr.iter().enumerate() {
          if let Some((j, entry_ptr)) = next_replacement {
            if *j == i {
              new_arr.push(StoragePoint::Ref(new_refs.len() as u64));
              new_refs.push(*entry_ptr);
              next_replacement = replacements_iter.next();
              continue;
            }
          }

          new_arr.push(point.clone());
        }

        Some(cache_and_store(
          tx,
          &StorageVal {
            point: StoragePoint::Array(Rc::new(new_arr)),
            refs: Rc::new(new_refs),
          },
          cache,
          cache_id(arr),
        )?)
      }
      StoragePoint::Ref(_) => None,
    })
  }

  fn cache_id(&self) -> Option<u64> {
    match &self {
      StoragePoint::Void => None,
      StoragePoint::Number(_) => None,
      StoragePoint::Array(arr) => Some(cache_id(arr)),
      StoragePoint::Ref(_) => None,
    }
  }

  #[cfg(test)]
  fn numbers<E, SO: StorageOps<E>>(
    &self,
    tx: &mut SO,
    refs: &Rc<Vec<StorageEntryPtr>>,
  ) -> Result<Vec<u64>, E> {
    match &self {
      StoragePoint::Void => Ok(Vec::new()),
      StoragePoint::Number(n) => Ok(vec![*n]),
      StoragePoint::Array(arr) => {
        let mut numbers = Vec::new();

        for point in arr.iter() {
          numbers.extend(point.numbers(tx, refs)?);
        }

        Ok(numbers)
      }
      StoragePoint::Ref(i) => {
        let key = refs[*i as usize];
        let val = tx.read(key)?.unwrap().to_val();
        val.point.numbers(tx, &val.refs)
      }
    }
  }
}

#[derive(Serialize, Deserialize)]
pub struct StorageEntry {
  pub(crate) ref_count: u64,

  #[serde(serialize_with = "serialize_rc", deserialize_with = "deserialize_rc")]
  pub(crate) refs: Rc<Vec<StorageEntryPtr>>,

  data: Vec<u8>,
}

impl StorageEntry {
  pub fn to_val(&self) -> StorageVal {
    StorageVal {
      point: bincode::deserialize(&self.data).unwrap(),
      refs: self.refs.clone(),
    }
  }
}

fn cache_id<T>(rc: &Rc<T>) -> u64 {
  rc.as_ref() as *const T as u64
}

fn cache_and_store<E, SO: StorageOps<E>>(
  tx: &mut SO,
  val: &StorageVal,
  cache: &mut HashMap<u64, StorageEntryPtr>,
  id: u64,
) -> Result<StorageEntryPtr, E> {
  let key = tx.store(val)?;

  let pre_existing = cache.insert(id, key);
  assert!(pre_existing.is_none());

  Ok(key)
}
