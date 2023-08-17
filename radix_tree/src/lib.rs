mod radix_tree;

pub use crate::radix_tree::RadixTree;

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn initial_len_0() {
    let tree = RadixTree::<usize, 4>::new();
    assert_eq!(tree.len(), 0);
  }

  #[test]
  fn push_once() {
    let mut tree = RadixTree::<usize, 4>::new();
    tree.push(0);
    assert_eq!(tree.len(), 1);
  }

  #[test]
  fn push_100() {
    let mut tree = RadixTree::<usize, 4>::new();

    for i in 0..100 {
      tree.push(i);
      assert_eq!(tree.len(), i + 1);
    }
  }
}
