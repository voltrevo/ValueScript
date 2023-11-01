#[cfg(test)]
mod tests_ {
  use std::rc::Rc;

  use crate::{
    demo_val::DemoVal, memory_backend::MemoryBackend, sled_backend::SledBackend, storage::Storage,
    storage_ptr::storage_head_ptr, StorageBackend,
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
        .set_head(storage_head_ptr(b"test"), &DemoVal::Number(123))
        .unwrap();

      let val = storage
        .get_head(storage_head_ptr(b"test"))
        .unwrap()
        .unwrap();

      if let DemoVal::Number(val) = val {
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
      let key0 = storage.store_tmp(&DemoVal::Number(0)).unwrap();
      let key1 = storage.store_tmp(&DemoVal::Number(1)).unwrap();

      storage
        .set_head(
          storage_head_ptr(b"test"),
          &DemoVal::Array(Rc::new(vec![DemoVal::Ptr(key0), DemoVal::Ptr(key1)])),
        )
        .unwrap();

      assert_eq!(storage.get_ref_count(key0).unwrap(), Some(2));
      assert_eq!(storage.get_ref_count(key1).unwrap(), Some(2));

      storage.clear_tmp().unwrap();

      assert_eq!(storage.get_ref_count(key0).unwrap(), Some(1));
      assert_eq!(storage.get_ref_count(key1).unwrap(), Some(1));

      storage.remove_head(storage_head_ptr(b"test")).unwrap();

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
          &DemoVal::Array(Rc::new(vec![
            DemoVal::Array(Rc::new(vec![DemoVal::Number(1), DemoVal::Number(2)])),
            DemoVal::Array(Rc::new(vec![DemoVal::Number(3), DemoVal::Number(4)])),
          ])),
        )
        .unwrap();

      let value = storage
        .get_head::<DemoVal>(storage_head_ptr(b"test"))
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
      let mut arr = DemoVal::Array(Rc::new(vec![DemoVal::Number(123)]));

      let depth = 3;

      for _ in 0..depth {
        arr = DemoVal::Array(Rc::new(vec![arr.clone(), arr]));
      }

      storage.set_head(storage_head_ptr(b"test"), &arr).unwrap();

      let value = storage
        .get_head::<DemoVal>(storage_head_ptr(b"test"))
        .unwrap()
        .unwrap();

      assert_eq!(
        value.numbers(storage).unwrap(),
        vec![123; 2usize.pow(depth as u32)]
      );

      storage.remove_head(storage_head_ptr(b"test")).unwrap();

      assert_eq!(storage.sb.borrow().len(), 0);
      assert!(storage.is_empty());
    }

    run(impl_, impl_);
  }

  #[test]
  fn large_redundant_tree() {
    fn impl_<SB: StorageBackend>(storage: &mut Storage<SB>) {
      let mut arr = DemoVal::Array(Rc::new(vec![DemoVal::Number(123)]));

      // 2^100 = 1267650600228229401496703205376
      // This tests that we can handle a tree with 2^100 nodes.
      // We do this by reusing the same array as both children of the parent array.
      // It's particularly important that this also happens during storage.
      for _ in 0..100 {
        arr = DemoVal::Array(Rc::new(vec![arr.clone(), arr]));
      }

      storage.set_head(storage_head_ptr(b"test"), &arr).unwrap();

      let value = storage
        .get_head(storage_head_ptr(b"test"))
        .unwrap()
        .unwrap();

      if let DemoVal::Array(arr) = value {
        assert_eq!(arr.len(), 2);
      } else {
        panic!("Expected array");
      }

      storage.remove_head(storage_head_ptr(b"test")).unwrap();

      assert!(storage.is_empty());
    }

    run(impl_, impl_);
  }
}
