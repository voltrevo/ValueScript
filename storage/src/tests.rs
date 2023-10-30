#[cfg(test)]
mod tests_ {
  use std::rc::Rc;

  use crate::{
    memory_backend::MemoryBackend,
    sled_backend::SledBackend,
    storage::Storage,
    storage_ptr::storage_head_ptr,
    storage_val::{StorageArray, StorageCompoundVal, StorageVal},
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
        .set_head(storage_head_ptr(b"test"), Some(&StorageVal::Number(123)))
        .unwrap();

      let val = storage
        .get_head(storage_head_ptr(b"test"))
        .unwrap()
        .unwrap();

      if let StorageVal::Number(val) = val {
        assert_eq!(val, 123);
      } else {
        panic!("Expected number");
      }
    }

    run(impl_, impl_);
  }

  #[test]
  fn array_0_1() {
    fn impl_<SB: StorageBackend>(storage: &mut Storage<SB>) {
      let key0 = storage.store_tmp(&StorageVal::Number(0)).unwrap();
      let key1 = storage.store_tmp(&StorageVal::Number(1)).unwrap();

      storage
        .set_head(
          storage_head_ptr(b"test"),
          Some(&StorageVal::Compound(Rc::new(StorageCompoundVal::Array(
            StorageArray {
              items: vec![StorageVal::Ptr(key0), StorageVal::Ptr(key1)],
            },
          )))),
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
          Some(&StorageVal::Compound(Rc::new(StorageCompoundVal::Array(
            StorageArray {
              items: vec![
                StorageVal::Compound(Rc::new(StorageCompoundVal::Array(StorageArray {
                  items: vec![StorageVal::Number(1), StorageVal::Number(2)],
                }))),
                StorageVal::Compound(Rc::new(StorageCompoundVal::Array(StorageArray {
                  items: vec![StorageVal::Number(3), StorageVal::Number(4)],
                }))),
              ],
            },
          )))),
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
      let mut arr = StorageVal::Compound(Rc::new(StorageCompoundVal::Array(StorageArray {
        items: vec![StorageVal::Number(123)],
      })));

      let depth = 3;

      for _ in 0..depth {
        arr = StorageVal::Compound(Rc::new(StorageCompoundVal::Array(StorageArray {
          items: vec![arr.clone(), arr],
        })));
      }

      storage
        .set_head(storage_head_ptr(b"test"), Some(&arr))
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
      let mut arr = StorageVal::Compound(Rc::new(StorageCompoundVal::Array(StorageArray {
        items: vec![StorageVal::Number(123)],
      })));

      // 2^100 = 1267650600228229401496703205376
      // This tests that we can handle a tree with 2^100 nodes.
      // We do this by reusing the same array as both children of the parent array.
      // It's particularly important that this also happens during storage.
      for _ in 0..100 {
        arr = StorageVal::Compound(Rc::new(StorageCompoundVal::Array(StorageArray {
          items: vec![arr.clone(), arr],
        })));
      }

      storage
        .set_head(storage_head_ptr(b"test"), Some(&arr))
        .unwrap();

      let value = storage
        .get_head(storage_head_ptr(b"test"))
        .unwrap()
        .unwrap();

      if let StorageVal::Compound(compound) = value {
        let StorageCompoundVal::Array(arr) = &*compound;
        assert_eq!(arr.items.len(), 2);
      } else {
        panic!("Expected array");
      }

      storage.set_head(storage_head_ptr(b"test"), None).unwrap();

      assert!(storage.is_empty());
    }

    run(impl_, impl_);
  }
}
