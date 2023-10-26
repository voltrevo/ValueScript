#[cfg(test)]
mod tests_ {
  use crate::{
    memory_backend::MemoryBackend,
    sled_backend::SledBackend,
    storage::{Storage, StorageBackend},
  };

  fn run(impl_memory: fn(&mut Storage<MemoryBackend>), impl_sled: fn(&mut Storage<SledBackend>)) {
    let mut storage = Storage::new(MemoryBackend::new());
    impl_memory(&mut storage);

    let mut storage = Storage::new(SledBackend::open_in_memory().unwrap());
    impl_sled(&mut storage);
  }

  #[test]
  fn number() {
    fn impl_<SB: StorageBackend>(storage: &mut Storage<SB>) {
      let key = storage.write_number(123.456).unwrap();
      assert_eq!(storage.read_number(key).unwrap(), Some(123.456));
    }

    run(impl_, impl_);
  }
}
