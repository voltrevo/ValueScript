#[cfg(test)]
mod tests_ {
  use std::rc::Rc;

  use crate::{
    memory_backend::MemoryBackend,
    sled_backend::SledBackend,
    storage::Storage,
    storage_ptr::storage_head_ptr,
    storage_val::{StoragePoint, StorageVal},
    StorageBackend,
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

  #[test]
  fn array_of_arrays() {
    fn impl_<SB: StorageBackend>(storage: &mut Storage<SB>) {
      storage
        .set_head(
          storage_head_ptr(b"test"),
          Some(&StorageVal {
            point: StoragePoint::Array(Rc::new(vec![
              StoragePoint::Array(Rc::new(vec![
                StoragePoint::Number(1),
                StoragePoint::Number(2),
              ])),
              StoragePoint::Array(Rc::new(vec![
                StoragePoint::Number(3),
                StoragePoint::Number(4),
              ])),
            ])),
            refs: Rc::new(vec![]),
          }),
        )
        .unwrap();

      let value = storage
        .get_head(storage_head_ptr(b"test"))
        .unwrap()
        .unwrap();

      let numbers = value.numbers(storage).unwrap();

      assert_eq!(numbers, vec![1, 2, 3, 4]);
    }

    run(impl_, impl_);
  }

  #[test]
  fn small_redundant_tree() {
    fn impl_<SB: StorageBackend>(storage: &mut Storage<SB>) {
      let mut arr = StoragePoint::Array(Rc::new(vec![StoragePoint::Number(123)]));

      let depth = 3;

      for _ in 0..depth {
        arr = StoragePoint::Array(Rc::new(vec![arr.clone(), arr]));
      }

      storage
        .set_head(
          storage_head_ptr(b"test"),
          Some(&StorageVal {
            point: arr,
            refs: Rc::new(vec![]),
          }),
        )
        .unwrap();

      let value = storage
        .get_head(storage_head_ptr(b"test"))
        .unwrap()
        .unwrap();

      assert_eq!(
        value.numbers(storage).unwrap(),
        vec![123; 2usize.pow(depth as u32)]
      );

      storage.set_head(storage_head_ptr(b"test"), None).unwrap();

      assert_eq!(storage.sb.len(), 0);
      assert!(storage.is_empty());
    }

    run(impl_, impl_);
  }

  #[test]
  fn large_redundant_tree() {
    fn impl_<SB: StorageBackend>(storage: &mut Storage<SB>) {
      let mut arr = StoragePoint::Array(Rc::new(vec![StoragePoint::Number(123)]));

      // 2^100 = 1267650600228229401496703205376
      // This tests that we can handle a tree with 2^100 nodes.
      // We do this by reusing the same array as both children of the parent array.
      // It's particularly important that this also happens during storage.
      for _ in 0..100 {
        arr = StoragePoint::Array(Rc::new(vec![arr.clone(), arr]));
      }

      storage
        .set_head(
          storage_head_ptr(b"test"),
          Some(&StorageVal {
            point: arr,
            refs: Rc::new(vec![]),
          }),
        )
        .unwrap();

      let value = storage
        .get_head(storage_head_ptr(b"test"))
        .unwrap()
        .unwrap();

      if let StoragePoint::Array(arr) = value.point {
        assert_eq!(arr.len(), 2);
      } else {
        panic!("Expected array");
      }

      storage.set_head(storage_head_ptr(b"test"), None).unwrap();

      assert!(storage.is_empty());
    }

    run(impl_, impl_);
  }
}
