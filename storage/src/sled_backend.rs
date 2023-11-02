use std::{cell::RefCell, collections::HashMap, error::Error, fmt::Display, rc::Weak};

use sled::transaction::{
  ConflictableTransactionError, TransactionError, UnabortableTransactionError,
};

use crate::{
  rc_key::RcKey,
  storage_ptr::StorageEntryPtr,
  storage_tx::{StorageReader, StorageTxMut},
  GenericError, StorageBackend, StoragePtr,
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
  type TxMut<'a> = SledTxMut<'a>;

  fn transaction<F, T>(&self, self_weak: Weak<RefCell<Self>>, f: F) -> Result<T, Box<dyn Error>>
  where
    F: Fn(&mut Self::Tx<'_>) -> Result<T, GenericError>,
  {
    self
      .db
      .transaction(|tx| {
        let mut handle = SledTx {
          backend: self_weak.clone(),
          tx,
        };

        f(&mut handle).map_err(to_sled_conflictable_error)
      })
      .map_err(from_sled_tx_error)
  }

  fn transaction_mut<F, T>(
    &mut self,
    self_weak: Weak<RefCell<Self>>,
    f: F,
  ) -> Result<T, Box<dyn Error>>
  where
    F: Fn(&mut Self::TxMut<'_>) -> Result<T, GenericError>,
  {
    self
      .db
      .transaction(|tx| {
        let mut handle = SledTxMut {
          backend: self_weak.clone(),
          ref_deltas: Default::default(),
          cache: Default::default(),
          tx,
        };

        let res = f(&mut handle).map_err(to_sled_conflictable_error)?;

        handle
          .flush_ref_deltas()
          .map_err(to_sled_conflictable_error)?;

        Ok(res)
      })
      .map_err(from_sled_tx_error)
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
  backend: Weak<RefCell<SledBackend>>,
  tx: &'a sled::transaction::TransactionalTree,
}

impl<'a> StorageReader<'a, SledBackend> for SledTx<'a> {
  fn read_bytes<T>(&self, ptr: StoragePtr<T>) -> Result<Option<Vec<u8>>, GenericError> {
    let value = self
      .tx
      .get(ptr.to_bytes())
      .map_err(from_sled_unabortable_error)?
      .map(|value| value.to_vec());

    Ok(value)
  }

  fn get_backend(&self) -> Weak<RefCell<SledBackend>> {
    self.backend.clone()
  }
}

pub struct SledTxMut<'a> {
  backend: Weak<RefCell<SledBackend>>,
  ref_deltas: HashMap<(u64, u64, u64), i64>,
  cache: HashMap<RcKey, StorageEntryPtr>,
  tx: &'a sled::transaction::TransactionalTree,
}

impl<'a> StorageReader<'a, SledBackend> for SledTxMut<'a> {
  fn read_bytes<T>(&self, ptr: StoragePtr<T>) -> Result<Option<Vec<u8>>, GenericError> {
    let value = self
      .tx
      .get(ptr.to_bytes())
      .map_err(from_sled_unabortable_error)?
      .map(|value| value.to_vec());

    Ok(value)
  }

  fn get_backend(&self) -> Weak<RefCell<SledBackend>> {
    self.backend.clone()
  }
}

impl<'a> StorageTxMut<'a, SledBackend> for SledTxMut<'a> {
  fn ref_deltas(&mut self) -> &mut HashMap<(u64, u64, u64), i64> {
    &mut self.ref_deltas
  }

  fn cache(&mut self) -> &mut HashMap<RcKey, StorageEntryPtr> {
    &mut self.cache
  }

  fn write_bytes<T>(
    &mut self,
    ptr: StoragePtr<T>,
    data: Option<Vec<u8>>,
  ) -> Result<(), GenericError> {
    match data {
      Some(data) => self
        .tx
        .insert(ptr.to_bytes(), data)
        .map_err(from_sled_unabortable_error)?,
      None => self
        .tx
        .remove(ptr.to_bytes())
        .map_err(from_sled_unabortable_error)?,
    };

    Ok(())
  }
}

fn to_sled_conflictable_error(e: Box<dyn Error>) -> ConflictableTransactionError<GenericError> {
  match e.downcast::<GenericSledError>() {
    Ok(e) => e.0,
    Err(e) => ConflictableTransactionError::Abort(e),
  }
}

fn from_sled_tx_error(e: TransactionError<GenericError>) -> GenericError {
  match e {
    TransactionError::Abort(e) => e,
    TransactionError::Storage(e) => e.into(),
  }
}

fn from_sled_unabortable_error(e: UnabortableTransactionError) -> GenericError {
  let conflictable = Into::<ConflictableTransactionError<GenericError>>::into(e);

  GenericSledError(conflictable).into()
}

#[derive(Debug)]
pub struct GenericSledError(pub sled::transaction::ConflictableTransactionError<GenericError>);

impl Error for GenericSledError {}

impl Display for GenericSledError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.0)
  }
}

impl From<sled::transaction::ConflictableTransactionError<GenericError>> for GenericSledError {
  fn from(e: sled::transaction::ConflictableTransactionError<GenericError>) -> Self {
    Self(e)
  }
}
