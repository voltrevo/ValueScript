use std::any::Any;
use std::hash::{Hash, Hasher};
use std::rc::Rc;

/**
 * A wrapper around Rc suitable for use as a key in a HashMap.
 *
 * This fixes the issue with using the pointer as the key, since the pointer might be reused. By
 * using RcKey, we still effectively use the pointer as the key, but we also hold a reference to the
 * rc, so it cannot be dropped and reused, which would create a false association.
 */
pub struct RcKey(Rc<dyn Any>);

impl RcKey {
  pub fn from<T: 'static>(value: Rc<T>) -> Self {
    RcKey(value as Rc<dyn Any>)
  }
}

impl PartialEq for RcKey {
  fn eq(&self, other: &Self) -> bool {
    // This version:
    //   Rc::ptr_eq(&self.0, &other.0)
    //
    // results in this feedback from clippy:
    //   comparing trait object pointers compares a non-unique vtable address
    //   consider extracting and comparing data pointers only
    //
    // The implementation below is an attempt to follow the advice of comparing data pointers, but
    // I'm not sure whether this is a correct fix.
    //
    // For our purposes (caching), I suspect that even if this issue isn't resolved it is
    // acceptable, since an alternative address would only reduce the cache hits, not cause
    // incorrect behavior.

    let self_ptr = Rc::as_ptr(&self.0) as *const ();
    let other_ptr = Rc::as_ptr(&other.0) as *const ();

    self_ptr == other_ptr
  }
}

impl Eq for RcKey {}

impl Hash for RcKey {
  fn hash<H: Hasher>(&self, state: &mut H) {
    Rc::as_ptr(&self.0).hash(state);
  }
}
