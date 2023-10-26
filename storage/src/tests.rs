#[cfg(test)]
mod tests_ {
  use std::rc::Rc;

  use crate::{
    memory_backend::MemoryBackend,
    sled_backend::SledBackend,
    storage::{Storage, StorageBackend, StoragePoint, StorageVal},
    storage_ptr::storage_head_ptr,
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
          storage_head_ptr(b"test"),
          Some(&StorageVal {
            point: StoragePoint::Number(123),
            refs: Rc::new(vec![]),
          }),
        )
        .unwrap();

      let val = storage
        .get_head(storage_head_ptr(b"test"))
        .unwrap()
        .unwrap();

      assert_eq!(val.point, StoragePoint::Number(123));
    }

    run(impl_, impl_);
  }

  #[test]
  fn array_0_1() {
    fn impl_<SB: StorageBackend>(storage: &mut Storage<SB>) {
      let key0 = storage
        .store_tmp(&StorageVal {
          point: StoragePoint::Number(0),
          refs: Rc::new(vec![]),
        })
        .unwrap();

      let key1 = storage
        .store_tmp(&StorageVal {
          point: StoragePoint::Number(1),
          refs: Rc::new(vec![]),
        })
        .unwrap();

      storage
        .set_head(
          storage_head_ptr(b"test"),
          Some(&StorageVal {
            point: StoragePoint::Array(Rc::new(vec![StoragePoint::Ref(0), StoragePoint::Ref(1)])),
            refs: Rc::new(vec![key0, key1]),
          }),
        )
        .unwrap();

      assert_eq!(storage.get_ref_count(key0).unwrap(), Some(2));
      assert_eq!(storage.get_ref_count(key1).unwrap(), Some(2));

      storage.clear_tmp().unwrap();

      assert_eq!(storage.get_ref_count(key0).unwrap(), Some(1));
      assert_eq!(storage.get_ref_count(key1).unwrap(), Some(1));

      storage.set_head(storage_head_ptr(b"test"), None).unwrap();

      assert_eq!(storage.get_ref_count(key0).unwrap(), None);
      assert_eq!(storage.get_ref_count(key1).unwrap(), None);

      assert!(storage.is_empty());
    }

    run(impl_, impl_);
  }
}
