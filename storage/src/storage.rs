use std::{fmt::Debug as DebugTrait, rc::Rc};

use rand::thread_rng;
use serde::{Deserialize, Serialize};

use crate::serde_rc::{deserialize_rc, serialize_rc};
use crate::storage_ptr::{tmp_at_ptr, tmp_count_ptr, StorageEntryPtr, StorageHeadPtr, StoragePtr};

pub struct Storage<SB: StorageBackend> {
  sb: SB,
}

pub trait StorageBackendHandle<'a, E> {
  fn read_bytes<T>(&self, key: StoragePtr<T>) -> Result<Option<Vec<u8>>, E>;
  fn write_bytes<T>(&mut self, key: StoragePtr<T>, data: Option<Vec<u8>>) -> Result<(), E>;
}

pub trait StorageOps<E> {
  fn read<T: for<'de> Deserialize<'de>>(&mut self, key: StoragePtr<T>) -> Result<Option<T>, E>;
  fn write<T: Serialize>(&mut self, key: StoragePtr<T>, data: Option<&T>) -> Result<(), E>;

  fn inc_ref(&mut self, key: StorageEntryPtr) -> Result<(), E>;
  fn dec_ref(&mut self, key: StorageEntryPtr) -> Result<(), E>;

  fn get_head(&mut self, ptr: StorageHeadPtr) -> Result<Option<StorageVal>, E>;
  fn set_head(&mut self, ptr: StorageHeadPtr, value: Option<&StorageVal>) -> Result<(), E>;

  fn store_with_replacements(&mut self, value: &StorageVal) -> Result<StorageEntryPtr, E>;
  fn store(&mut self, value: &StorageVal) -> Result<StorageEntryPtr, E>;
}

impl<'a, Handle, E> StorageOps<E> for Handle
where
  Handle: StorageBackendHandle<'a, E>,
{
  fn read<T: for<'de> Deserialize<'de>>(&mut self, key: StoragePtr<T>) -> Result<Option<T>, E> {
    let data = match self.read_bytes(key)? {
      Some(data) => data,
      None => return Ok(None),
    };

    Ok(Some(bincode::deserialize(&data).unwrap()))
  }

  fn write<T: Serialize>(&mut self, key: StoragePtr<T>, data: Option<&T>) -> Result<(), E> {
    self.write_bytes(key, data.map(|data| bincode::serialize(&data).unwrap()))
  }

  fn inc_ref(&mut self, key: StorageEntryPtr) -> Result<(), E> {
    let mut entry = match self.read(key)? {
      Some(entry) => entry,
      None => panic!("Key does not exist"),
    };

    entry.ref_count += 1;

    self.write(key, Some(&entry))
  }

  fn dec_ref(&mut self, key: StorageEntryPtr) -> Result<(), E> {
    let mut entry = match self.read(key)? {
      Some(entry) => entry,
      None => panic!("Key does not exist"),
    };

    entry.ref_count -= 1;

    if entry.ref_count == 0 {
      for key in entry.refs.iter() {
        self.dec_ref(*key)?;
      }

      self.write(key, None)
    } else {
      self.write(key, Some(&entry))
    }
  }

  fn get_head(&mut self, ptr: StorageHeadPtr) -> Result<Option<StorageVal>, E> {
    let key = match self.read(ptr)? {
      Some(key) => key,
      None => return Ok(None),
    };

    let value = self.read(key)?.map(|entry| entry.to_val());

    Ok(value)
  }

  fn set_head(&mut self, ptr: StorageHeadPtr, value: Option<&StorageVal>) -> Result<(), E> {
    if let Some(value) = value {
      let key = self.store_with_replacements(value)?;

      // TODO: Performance: Identify overlapping keys and cancel out inc+dec
      if let Some(old_key) = self.read(ptr)? {
        self.dec_ref(old_key)?;
      }

      self.write(ptr, Some(&key))
    } else {
      if let Some(old_key) = self.read(ptr)? {
        self.dec_ref(old_key)?;
      }

      self.write(ptr, None)
    }
  }

  fn store_with_replacements(&mut self, value: &StorageVal) -> Result<StorageEntryPtr, E> {
    if let Some(key) = value.point.maybe_replace_store(self, &value.refs)? {
      return Ok(key);
    }

    self.store(value)
  }

  fn store(&mut self, value: &StorageVal) -> Result<StorageEntryPtr, E> {
    let key = StoragePtr::random(&mut thread_rng());
    self.write(key, Some(&value.to_entry()))?;

    for subkey in value.refs.iter() {
      self.inc_ref(*subkey)?;
    }

    Ok(key)
  }
}

pub trait StorageBackend {
  type Error<E: DebugTrait>: DebugTrait;
  type InTransactionError<E>;
  type Handle<'a, E>: StorageBackendHandle<'a, Self::InTransactionError<E>>;

  fn transaction<F, T, E: DebugTrait>(&mut self, f: F) -> Result<T, Self::Error<E>>
  where
    F: Fn(&mut Self::Handle<'_, E>) -> Result<T, Self::InTransactionError<E>>;

  fn is_empty(&self) -> bool;
}

impl<SB: StorageBackend> Storage<SB> {
  pub fn new(sb: SB) -> Self {
    Self { sb }
  }

  pub fn get_head(&mut self, ptr: StorageHeadPtr) -> Result<Option<StorageVal>, SB::Error<()>> {
    self.sb.transaction(|sb| sb.get_head(ptr))
  }

  pub fn set_head(
    &mut self,
    ptr: StorageHeadPtr,
    value: Option<&StorageVal>,
  ) -> Result<(), SB::Error<()>> {
    self.sb.transaction(|sb| sb.set_head(ptr, value))
  }

  pub fn store_tmp(&mut self, value: &StorageVal) -> Result<StorageEntryPtr, SB::Error<()>> {
    self.sb.transaction(|sb| {
      let tmp_count = sb.read(tmp_count_ptr())?.unwrap_or(0);
      let tmp_ptr = tmp_at_ptr(tmp_count);
      sb.set_head(tmp_ptr, Some(value))?;

      sb.write(tmp_count_ptr(), Some(&(tmp_count + 1)))?;

      let key = sb
        .read(tmp_ptr)?
        .unwrap_or_else(|| panic!("Missing tmp key"));

      Ok(key)
    })
  }

  pub fn clear_tmp(&mut self) -> Result<(), SB::Error<()>> {
    self.sb.transaction(|sb| {
      let tmp_count = sb.read(tmp_count_ptr())?.unwrap_or(0);

      for i in 0..tmp_count {
        sb.set_head(tmp_at_ptr(i), None)?;
      }

      sb.write(tmp_count_ptr(), None)?;

      Ok(())
    })
  }

  pub fn is_empty(&self) -> bool {
    self.sb.is_empty()
  }

  pub(crate) fn get_ref_count(
    &mut self,
    key: StorageEntryPtr,
  ) -> Result<Option<u64>, SB::Error<()>> {
    self
      .sb
      .transaction(|sb| Ok(sb.read(key)?.map(|entry| entry.ref_count)))
  }
}

#[derive(Serialize, Deserialize)]
pub struct StorageEntry {
  ref_count: u64,

  #[serde(serialize_with = "serialize_rc", deserialize_with = "deserialize_rc")]
  refs: Rc<Vec<StorageEntryPtr>>,

  data: Vec<u8>,
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
  ) -> Result<Option<StorageEntryPtr>, E> {
    Ok(match &self {
      StoragePoint::Void => None,
      StoragePoint::Number(_) => None,
      StoragePoint::Array(arr) => 'b: {
        let mut replacements: Vec<(usize, StorageEntryPtr)> = Vec::new();

        for i in 0..arr.len() {
          if let Some(key) = arr[i].maybe_replace_store(tx, refs)? {
            replacements.push((i, key));
          }
        }

        if replacements.is_empty() {
          break 'b Some(tx.store(&StorageVal {
            point: StoragePoint::Array(arr.clone()),
            refs: refs.clone(),
          })?);
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

        Some(tx.store(&StorageVal {
          point: StoragePoint::Array(Rc::new(new_arr)),
          refs: Rc::new(new_refs),
        })?)
      }
      StoragePoint::Ref(_) => None,
    })
  }

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

#[derive(Serialize, Deserialize, Debug)]
pub struct StorageVal {
  pub point: StoragePoint,

  #[serde(serialize_with = "serialize_rc", deserialize_with = "deserialize_rc")]
  pub refs: Rc<Vec<StorageEntryPtr>>,
}

impl StorageEntry {
  pub fn to_val(&self) -> StorageVal {
    StorageVal {
      point: bincode::deserialize(&self.data).unwrap(),
      refs: self.refs.clone(),
    }
  }
}

impl StorageVal {
  pub fn to_entry(&self) -> StorageEntry {
    StorageEntry {
      ref_count: 1,
      refs: self.refs.clone(),
      data: bincode::serialize(&self.point).unwrap(),
    }
  }

  pub(crate) fn numbers<SB: StorageBackend>(
    &self,
    storage: &mut Storage<SB>,
  ) -> Result<Vec<u64>, SB::Error<()>> {
    storage
      .sb
      .transaction(|sb| self.point.numbers(sb, &self.refs))
  }
}
