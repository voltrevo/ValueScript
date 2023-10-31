use std::{collections::HashMap, error::Error};

use crate::{
  rc_key::RcKey, storage_backend::StorageError, storage_ptr::StorageEntryPtr,
  storage_tx::StorageTx, StorageBackend, StoragePtr,
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
  type CustomError = sled::transaction::ConflictableTransactionError<Box<dyn Error>>;
  type Tx<'a> = SledTx<'a>;

  fn transaction<F, T>(&mut self, f: F) -> Result<T, Box<dyn Error>>
  where
    F: Fn(&mut Self::Tx<'_>) -> Result<T, StorageError<Self>>,
  {
    self
      .db
      .transaction(|tx| {
        let mut handle = SledTx {
          ref_deltas: Default::default(),
          cache: Default::default(),
          tx,
        };

        let res = f(&mut handle).map_err(|e| match e {
          StorageError::CustomError(e) => e,
          StorageError::Error(e) => Self::CustomError::Abort(e),
        })?;

        handle.flush_ref_deltas().map_err(|e| match e {
          StorageError::CustomError(e) => e,
          StorageError::Error(e) => Self::CustomError::Abort(e),
        })?;

        Ok(res)
      })
      .map_err(|e| match e {
        sled::transaction::TransactionError::Abort(e) => e,
        sled::transaction::TransactionError::Storage(e) => e.into(),
      })
  }

  fn is_empty(&self) -> bool {
    self.db.is_empty()
  }

  #[cfg(test)]
  fn len(&self) -> usize {
    self.db.len()
  }
}

pub struct SledTx<'a> {
  ref_deltas: HashMap<(u64, u64, u64), i64>,
  cache: HashMap<RcKey, StorageEntryPtr>,
  tx: &'a sled::transaction::TransactionalTree,
}

impl<'a> StorageTx<'a, SledBackend> for SledTx<'a> {
  fn ref_deltas(&mut self) -> &mut HashMap<(u64, u64, u64), i64> {
    &mut self.ref_deltas
  }

  fn cache(&mut self) -> &mut HashMap<RcKey, StorageEntryPtr> {
    &mut self.cache
  }

  fn read_bytes<T>(
    &self,
    ptr: StoragePtr<T>,
  ) -> Result<Option<Vec<u8>>, StorageError<SledBackend>> {
    let value = self
      .tx
      .get(ptr.to_bytes())
      .map_err(|e| StorageError::<SledBackend>::CustomError(e.into()))?
      .map(|value| value.to_vec());

    Ok(value)
  }

  fn write_bytes<T>(
    &mut self,
    ptr: StoragePtr<T>,
    data: Option<Vec<u8>>,
  ) -> Result<(), StorageError<SledBackend>> {
    match data {
      Some(data) => self
        .tx
        .insert(ptr.to_bytes(), data)
        .map_err(|e| StorageError::<SledBackend>::CustomError(e.into()))?,
      None => self
        .tx
        .remove(ptr.to_bytes())
        .map_err(|e| StorageError::<SledBackend>::CustomError(e.into()))?,
    };

    Ok(())
  }
}
