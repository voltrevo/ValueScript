use std::cell::RefCell;
use std::error::Error;
use std::rc::Rc;

use crate::storage_entity::StorageEntity;
use crate::storage_ptr::{tmp_at_ptr, tmp_count_ptr, StorageEntryPtr, StorageHeadPtr};
use crate::{StorageBackend, StorageReader, StorageTxMut};

pub struct Storage<SB: StorageBackend> {
  pub(crate) sb: Rc<RefCell<SB>>,
}

impl<SB: StorageBackend> Storage<SB> {
  pub fn new(sb: SB) -> Self {
    Self {
      sb: Rc::new(RefCell::new(sb)),
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

  pub fn clear_read_cache(&mut self) {
    self.sb.borrow_mut().get_read_cache().clear()
  }

  pub fn store_tmp<SE: StorageEntity<SB>>(
    &mut self,
    value: &SE,
  ) -> Result<StorageEntryPtr, Box<dyn Error>> {
    self
      .sb
      .borrow_mut()
      .transaction_mut(Rc::downgrade(&self.sb), |sb| {
        let tmp_count = sb.read(tmp_count_ptr())?.unwrap_or(0);
        let tmp_ptr = tmp_at_ptr(tmp_count);
        sb.set_head(tmp_ptr, value)?;

        sb.write(tmp_count_ptr(), Some(&(tmp_count + 1)))?;

        let ptr = sb.read(tmp_ptr)?.unwrap_or_else(|| panic!("Ptr not found"));

        Ok(ptr)
      })
  }

  pub fn clear_tmp(&mut self) -> Result<(), Box<dyn Error>> {
    self
      .sb
      .borrow_mut()
      .transaction_mut(Rc::downgrade(&self.sb), |sb| {
        let tmp_count = sb.read(tmp_count_ptr())?.unwrap_or(0);

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
    self
      .sb
      .read(ptr)
      .map(|entry| entry.map(|entry| entry.ref_count))
  }
}

impl<SB: StorageBackend> StorageReader<SB> for Storage<SB> {
  fn read_bytes<T>(
    &self,
    ptr: crate::StoragePtr<T>,
  ) -> Result<Option<Vec<u8>>, crate::GenericError> {
    self.sb.borrow().read_bytes(ptr)
  }

  fn get_backend(&self) -> std::rc::Weak<RefCell<SB>> {
    Rc::downgrade(&self.sb)
  }
}
