use arrayvec::ArrayVec;

use crate::{strict_radix_tree::StrictRadixTreeData, StrictRadixTree};

pub struct StrictRadixTreeIterator<'a, T, const N: usize> {
  meta_path: Vec<(&'a ArrayVec<StrictRadixTree<T, N>, N>, usize)>,
  leaf_path: (&'a ArrayVec<T, N>, usize),
}

impl<'a, T: Clone, const N: usize> StrictRadixTreeIterator<'a, T, N> {
  pub fn new(mut tree: &'a StrictRadixTree<T, N>) -> Self {
    let mut meta_path = Vec::<(&'a ArrayVec<StrictRadixTree<T, N>, N>, usize)>::new();

    loop {
      match tree.data() {
        StrictRadixTreeData::Meta(meta) => {
          meta_path.push((meta, 0));
          tree = &meta[0];
        }
        StrictRadixTreeData::Leaves(leaves) => {
          return Self {
            meta_path,
            leaf_path: (leaves, 0),
          };
        }
      }
    }
  }

  fn next_leaf(&mut self) -> Option<&'a T> {
    let (leaves, i) = &mut self.leaf_path;

    let res = leaves.get(*i);
    *i += 1;

    res
  }

  fn next_leaves(&mut self) -> Option<()> {
    for meta_i in (0..self.meta_path.len()).rev() {
      let (meta, i) = &mut self.meta_path[meta_i];
      *i += 1;

      if let Some(tree) = meta.get(*i) {
        self.set_path(meta_i + 1, tree);
        return Some(());
      }
    }

    None
  }

  fn set_path(&mut self, mut meta_i: usize, mut tree: &'a StrictRadixTree<T, N>) {
    loop {
      match tree.data() {
        StrictRadixTreeData::Meta(meta) => {
          self.meta_path[meta_i] = (meta, 0);
          meta_i += 1;
          tree = &meta[0];
        }
        StrictRadixTreeData::Leaves(leaves) => {
          self.leaf_path = (leaves, 0);
          break;
        }
      }
    }
  }
}

impl<'a, T: Clone, const N: usize> Iterator for StrictRadixTreeIterator<'a, T, N> {
  type Item = &'a T;

  fn next(&mut self) -> Option<Self::Item> {
    let leaf = self.next_leaf();

    match leaf {
      Some(_) => leaf,
      None => {
        self.next_leaves()?;
        self.next_leaf()
      }
    }
  }
}
