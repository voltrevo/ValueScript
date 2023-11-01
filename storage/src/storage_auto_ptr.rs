use std::fmt::{self, Debug};
use std::{cell::RefCell, error::Error, rc::Weak};

use crate::{StorageBackend, StorageEntity, StorageEntryPtr, StorageTx};

pub struct StorageAutoPtr<SB: StorageBackend, SE: for<'a> StorageEntity<'a, SB, SB::Tx<'a>>> {
  pub(crate) _marker: std::marker::PhantomData<SE>,
  pub(crate) sb: Weak<RefCell<SB>>, // TODO: Does this need to be weak?
  pub ptr: StorageEntryPtr,
}

impl<SB: StorageBackend, SE: for<'a> StorageEntity<'a, SB, SB::Tx<'a>>> Debug
  for StorageAutoPtr<SB, SE>
{
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("StorageAutoPtr")
      .field("ptr", &self.ptr)
      .finish()
  }
}

impl<SB: StorageBackend, SE: for<'a> StorageEntity<'a, SB, SB::Tx<'a>>> StorageAutoPtr<SB, SE> {
  pub fn resolve(&self) -> Result<Option<SE>, Box<dyn Error>> {
    let sb = match self.sb.upgrade() {
      Some(sb) => sb,
      None => return Err("Storage backend dropped".into()),
    };

    let res = sb.borrow_mut().transaction(self.sb.clone(), |sb| {
      Ok(match sb.read(self.ptr)? {
        Some(entry) => Some(SE::from_storage_entry(sb, entry)?),
        None => None,
      })
    });

    res
  }
}
