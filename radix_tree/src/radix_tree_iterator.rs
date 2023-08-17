use arrayvec::ArrayVec;

use crate::{radix_tree::RadixTreeData, RadixTree};

pub struct RadixTreeIterator<'a, T, const N: usize> {
  meta_path: Vec<(&'a ArrayVec<RadixTree<T, N>, N>, usize)>,
  leaf_path: (&'a ArrayVec<T, N>, usize),
}

impl<'a, T: Clone, const N: usize> RadixTreeIterator<'a, T, N> {
  pub fn new(mut tree: &'a RadixTree<T, N>) -> Self {
    let mut meta_path = Vec::<(&'a ArrayVec<RadixTree<T, N>, N>, usize)>::new();

    loop {
      match tree.data() {
        RadixTreeData::Meta(meta) => {
          meta_path.push((meta, 0));
          tree = &meta[0];
        }
        RadixTreeData::Leaves(leaves) => {
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

  fn set_path(&mut self, mut meta_i: usize, mut tree: &'a RadixTree<T, N>) {
    loop {
      match tree.data() {
        RadixTreeData::Meta(meta) => {
          self.meta_path[meta_i] = (meta, 0);
          meta_i += 1;
          tree = &meta[0];
        }
        RadixTreeData::Leaves(leaves) => {
          self.leaf_path = (leaves, 0);
          break;
        }
      }
    }
  }
}

impl<'a, T: Clone, const N: usize> Iterator for RadixTreeIterator<'a, T, N> {
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
