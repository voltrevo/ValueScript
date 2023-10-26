use std::fmt::Debug as DebugTrait;

use crate::{
  storage::{StorageBackend, StorageBackendHandle},
  StoragePtr,
};

pub struct SledBackend {
  db: sled::Db,
}

impl SledBackend {
  pub fn open<P>(path: P) -> Result<Self, sled::Error>
  where
    P: AsRef<std::path::Path>,
  {
    Ok(Self {
      db: sled::open(path)?,
    })
  }

  pub fn open_in_memory() -> Result<Self, sled::Error> {
    Ok(Self {
      db: sled::Config::new().temporary(true).open()?,
    })
  }
}

impl StorageBackend for SledBackend {
  type Error<E: DebugTrait> = sled::transaction::TransactionError<E>;
  type InTransactionError<E> = sled::transaction::ConflictableTransactionError<E>;
  type Handle<'a, E> = SledBackendHandle<'a>;

  fn transaction<F, T, E: DebugTrait>(&mut self, f: F) -> Result<T, Self::Error<E>>
  where
    F: Fn(&mut Self::Handle<'_, E>) -> Result<T, Self::InTransactionError<E>>,
  {
    self.db.transaction(|tx| {
      let mut handle = SledBackendHandle { tx };
      f(&mut handle)
    })
  }
}

pub struct SledBackendHandle<'a> {
  tx: &'a sled::transaction::TransactionalTree,
}

impl<'a, E> StorageBackendHandle<'a, sled::transaction::ConflictableTransactionError<E>>
  for SledBackendHandle<'a>
{
  fn read_bytes<T>(
    &self,
    key: StoragePtr<T>,
  ) -> Result<Option<Vec<u8>>, sled::transaction::ConflictableTransactionError<E>> {
    let value = self.tx.get(key.to_bytes())?.map(|value| value.to_vec());
    Ok(value)
  }

  fn write_bytes<T>(
    &mut self,
    key: StoragePtr<T>,
    data: Option<Vec<u8>>,
  ) -> Result<(), sled::transaction::ConflictableTransactionError<E>> {
    match data {
      Some(data) => self.tx.insert(key.to_bytes(), data)?,
      None => self.tx.remove(key.to_bytes())?,
    };

    Ok(())
  }
}
