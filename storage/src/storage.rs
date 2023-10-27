use crate::storage_ops::StorageOps;
use crate::storage_ptr::{tmp_at_ptr, tmp_count_ptr, StorageEntryPtr, StorageHeadPtr};
use crate::storage_val::StorageVal;
use crate::StorageBackend;

pub struct Storage<SB: StorageBackend> {
  pub(crate) sb: SB,
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

  #[cfg(test)]
  pub(crate) fn get_ref_count(
    &mut self,
    key: StorageEntryPtr,
  ) -> Result<Option<u64>, SB::Error<()>> {
    self
      .sb
      .transaction(|sb| Ok(sb.read(key)?.map(|entry| entry.ref_count)))
  }
}
