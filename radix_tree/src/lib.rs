mod strict_radix_tree;
mod strict_radix_tree_iterator;

pub use crate::strict_radix_tree::StrictRadixTree;

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn empty_tree() {
    let tree = StrictRadixTree::<usize, 4>::new();
    assert!(tree.is_empty());
    assert_eq!(tree.len(), 0);
    assert_eq!(tree.first(), None);
    assert_eq!(tree.last(), None);
  }

  #[test]
  fn push_once() {
    let mut tree = StrictRadixTree::<usize, 4>::new();
    tree.push(0);
    assert_eq!(tree.len(), 1);
    assert!(!tree.is_empty());
  }

  #[test]
  fn push_100() {
    let mut tree = StrictRadixTree::<usize, 4>::new();

    for i in 0..100 {
      tree.push(i);
      assert_eq!(tree.len(), i + 1);
    }

    assert_eq!(tree.first(), Some(&0));
    assert_eq!(tree.last(), Some(&99));

    for i in 0..100 {
      assert_eq!(tree[i], i);
    }

    for i in 0..100 {
      tree[i] = 1000 + i;
    }

    for i in 0..100 {
      assert_eq!(tree[i], 1000 + i);
    }

    assert_eq!(tree.first(), Some(&1000));
    assert_eq!(tree.last(), Some(&1099));

    assert_eq!(tree.get(100), None);
    assert_eq!(tree.get_mut(100), None);
  }

  #[test]
  fn push_64() {
    let mut tree = StrictRadixTree::<usize, 4>::new();

    for i in 0..64 {
      tree.push(i);
      assert_eq!(tree.len(), i + 1);
    }

    for i in 0..64 {
      assert_eq!(tree.get(i), Some(&i));
      assert_eq!(tree.get_mut(i), Some(&mut i.clone()));
    }

    for i in 64..256 {
      assert_eq!(tree.get(i), None);
      assert_eq!(tree.get_mut(i), None);
    }
  }

  #[test]
  fn iters() {
    let mut tree = StrictRadixTree::<usize, 4>::new();

    for i in 0..100 {
      tree.push(i);
    }

    for (i, v) in tree.into_iter().enumerate() {
      assert_eq!(*v, i);
    }
  }

  #[test]
  fn pop_100() {
    let mut tree = StrictRadixTree::<usize, 4>::new();

    for i in 0..100 {
      tree.push(i);
    }

    assert_eq!(tree.depth(), 4);

    for i in (0..100).rev() {
      assert_eq!(tree.pop(), Some(i));
      assert_eq!(tree.len(), i);
    }

    assert_eq!(tree.pop(), None);
    assert_eq!(tree.depth(), 1);
  }

  #[test]
  fn truncate() {
    let mut tree = StrictRadixTree::<usize, 4>::new();

    for i in 0..100 {
      tree.push(i);
    }

    assert_eq!(tree.len(), 100);

    tree.truncate(100);
    assert_eq!(tree.len(), 100);

    tree.truncate(101);
    assert_eq!(tree.len(), 100);

    tree.truncate(1000);
    assert_eq!(tree.len(), 100);

    tree.truncate(50);
    assert_eq!(tree.len(), 50);
    assert_eq!(tree.depth(), 3);

    for i in 0..50 {
      assert_eq!(tree[i], i);
    }

    tree.truncate(1);
    assert_eq!(tree.len(), 1);
    assert_eq!(tree.depth(), 1);
  }
}
