mod memory_storage;
mod storage;

#[cfg(test)]
mod tests {
  use crate::{memory_storage::MemoryStorage, storage::Storage};

  #[test]
  fn number() {
    let mut storage = Storage::new(MemoryStorage::new());

    let key = storage.write_number(123.456);
    assert_eq!(storage.read_number(key), Some(123.456));
  }
}
