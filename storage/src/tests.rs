#[cfg(test)]
mod tests_ {
  use std::rc::Rc;

  use crate::{
    memory_backend::MemoryBackend,
    sled_backend::SledBackend,
    storage::{Storage, StorageBackend, StoragePoint, StorageVal},
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
      storage
        .set_head(
          b"test",
          Some(&StorageVal {
            point: StoragePoint::Number(123),
            refs: Rc::new(vec![]),
          }),
        )
        .unwrap();

      let val = storage.get_head(b"test").unwrap().unwrap();

      assert_eq!(val.point, StoragePoint::Number(123));
    }

    run(impl_, impl_);
  }
}
