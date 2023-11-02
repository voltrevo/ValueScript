use std::cell::RefCell;
use std::error::Error;
use std::rc::Rc;

use crate::storage_auto_ptr::StorageAutoPtr;
use crate::storage_entity::StorageEntity;
use crate::storage_ptr::{tmp_at_ptr, tmp_count_ptr, StorageEntryPtr, StorageHeadPtr};
use crate::{StorageBackend, StorageError, StorageTx, StorageTxMut};

pub struct Storage<SB: StorageBackend> {
  pub(crate) sb: Rc<RefCell<SB>>,
}

impl<SB: StorageBackend> Storage<SB> {
  pub fn new(sb: SB) -> Self {
    Self {
      sb: Rc::new(RefCell::new(sb)),
    }
  }

  pub fn get_head<SE: StorageEntity<SB>>(
    &mut self,
    ptr: StorageHeadPtr,
  ) -> Result<Option<SE>, Box<dyn Error>> {
    self
      .sb
      .borrow()
      .transaction(Rc::downgrade(&self.sb), |sb| sb.get_head(ptr))
  }

  pub fn get<SE: StorageEntity<SB>>(&self, ptr: StorageEntryPtr) -> Result<SE, Box<dyn Error>> {
    // TODO: Avoid going through a transaction when read-only
    self.sb.borrow().transaction(Rc::downgrade(&self.sb), |sb| {
      let entry = sb
        .read(ptr)?
        .ok_or(StorageError::Error("Ptr not found".into()))?;

      SE::from_storage_entry(sb, entry)
    })
  }

  pub fn get_auto_ptr<SE: StorageEntity<SB>>(
    &mut self,
    ptr: StorageEntryPtr,
  ) -> StorageAutoPtr<SB, SE> {
    StorageAutoPtr {
      _marker: std::marker::PhantomData,
      sb: Rc::downgrade(&self.sb),
      ptr,
    }
  }

  pub fn set_head<SE: StorageEntity<SB>>(
    &mut self,
    ptr: StorageHeadPtr,
    value: &SE,
  ) -> Result<(), Box<dyn Error>> {
    self
      .sb
      .borrow_mut()
      .transaction_mut(Rc::downgrade(&self.sb), |sb| sb.set_head(ptr, value))
  }

  pub fn remove_head(&mut self, ptr: StorageHeadPtr) -> Result<(), Box<dyn Error>> {
    self
      .sb
      .borrow_mut()
      .transaction_mut(Rc::downgrade(&self.sb), |sb| sb.remove_head(ptr))
  }

  pub fn store_tmp<SE: StorageEntity<SB>>(
    &mut self,
    value: &SE,
  ) -> Result<StorageEntryPtr, Box<dyn Error>> {
    self
      .sb
      .borrow_mut()
      .transaction_mut(Rc::downgrade(&self.sb), |sb| {
        let tmp_count = StorageTxMut::read(sb, tmp_count_ptr())?.unwrap_or(0);
        let tmp_ptr = tmp_at_ptr(tmp_count);
        sb.set_head(tmp_ptr, value)?;

        sb.write(tmp_count_ptr(), Some(&(tmp_count + 1)))?;

        let ptr = StorageTxMut::read(sb, tmp_ptr)?.unwrap_or_else(|| panic!("Ptr not found"));

        Ok(ptr)
      })
  }

  pub fn clear_tmp(&mut self) -> Result<(), Box<dyn Error>> {
    self
      .sb
      .borrow_mut()
      .transaction_mut(Rc::downgrade(&self.sb), |sb| {
        let tmp_count = StorageTxMut::read(sb, tmp_count_ptr())?.unwrap_or(0);

        for i in 0..tmp_count {
          sb.remove_head(tmp_at_ptr(i))?;
        }

        sb.write(tmp_count_ptr(), None)?;

        Ok(())
      })
  }

  pub fn is_empty(&self) -> bool {
    self.sb.borrow().is_empty()
  }

  #[cfg(test)]
  pub(crate) fn get_ref_count(
    &mut self,
    ptr: StorageEntryPtr,
  ) -> Result<Option<u64>, Box<dyn Error>> {
    self.sb.borrow().transaction(Rc::downgrade(&self.sb), |sb| {
      Ok(sb.read(ptr)?.map(|entry| entry.ref_count))
    })
  }
}
