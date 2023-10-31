use crate::storage_backend_handle::StorageBackendHandle;
use crate::storage_entity::StorageEntity;
use crate::storage_ptr::{tmp_at_ptr, tmp_count_ptr, StorageEntryPtr, StorageHeadPtr};
use crate::StorageBackend;

pub struct Storage<SB: StorageBackend> {
  pub(crate) sb: SB,
}

impl<SB: StorageBackend> Storage<SB> {
  pub fn new(sb: SB) -> Self {
    Self { sb }
  }

  pub fn get_head<SE: for<'a> StorageEntity<'a, SB::InTransactionError<()>, SB::Handle<'a, ()>>>(
    &mut self,
    ptr: StorageHeadPtr,
  ) -> Result<Option<SE>, SB::Error<()>> {
    self.sb.transaction(|sb| sb.get_head(ptr))
  }

  pub fn set_head<SE: for<'a> StorageEntity<'a, SB::InTransactionError<()>, SB::Handle<'a, ()>>>(
    &mut self,
    ptr: StorageHeadPtr,
    value: &SE,
  ) -> Result<(), SB::Error<()>> {
    self.sb.transaction(|sb| sb.set_head(ptr, value))
  }

  pub fn remove_head(&mut self, ptr: StorageHeadPtr) -> Result<(), SB::Error<()>> {
    self.sb.transaction(|sb| sb.remove_head(ptr))
  }

  pub fn store_tmp<
    SE: for<'a> StorageEntity<'a, SB::InTransactionError<()>, SB::Handle<'a, ()>>,
  >(
    &mut self,
    value: &SE,
  ) -> Result<StorageEntryPtr, SB::Error<()>> {
    self.sb.transaction(|sb| {
      let tmp_count = sb.read(tmp_count_ptr())?.unwrap_or(0);
      let tmp_ptr = tmp_at_ptr(tmp_count);
      sb.set_head(tmp_ptr, value)?;

      sb.write(tmp_count_ptr(), Some(&(tmp_count + 1)))?;

      let ptr = sb.read(tmp_ptr)?.unwrap_or_else(|| panic!("Ptr not found"));

      Ok(ptr)
    })
  }

  pub fn clear_tmp(&mut self) -> Result<(), SB::Error<()>> {
    self.sb.transaction(|sb| {
      let tmp_count = sb.read(tmp_count_ptr())?.unwrap_or(0);

      for i in 0..tmp_count {
        sb.remove_head(tmp_at_ptr(i))?;
      }

      sb.write(tmp_count_ptr(), None)?;

      Ok(())
    })
  }

  pub fn is_empty(&self) -> bool {
    self.sb.is_empty()
  }

  #[cfg(test)]
  pub(crate) fn get_ref_count(
    &mut self,
    ptr: StorageEntryPtr,
  ) -> Result<Option<u64>, SB::Error<()>> {
    self
      .sb
      .transaction(|sb| Ok(sb.read(ptr)?.map(|entry| entry.ref_count)))
  }
}
