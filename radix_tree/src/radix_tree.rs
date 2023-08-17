use std::{mem::swap, rc::Rc};

use arrayvec::ArrayVec;

#[derive(Clone)]
pub enum RadixTreeData<T, const N: usize> {
  Meta(ArrayVec<RadixTree<T, N>, N>),
  Leaves(ArrayVec<T, N>),
}

#[derive(Clone)]
pub struct RadixTree<T, const N: usize>(Rc<RadixTreeData<T, N>>);

impl<T: Clone, const N: usize> RadixTree<T, N> {
  pub fn new() -> Self {
    RadixTree::<T, N>(Rc::new(RadixTreeData::<T, N>::Leaves(ArrayVec::new())))
  }

  fn new_meta() -> Self {
    RadixTree::<T, N>(Rc::new(RadixTreeData::<T, N>::Meta(ArrayVec::new())))
  }

  pub fn is_empty(&self) -> bool {
    match &*self.0 {
      RadixTreeData::Meta(_) => false,
      RadixTreeData::Leaves(leaves) => leaves.is_empty(),
    }
  }

  pub fn len(&self) -> usize {
    let mut res = 0;
    let mut tree = self;

    loop {
      match &*tree.0 {
        RadixTreeData::Meta(meta) => {
          let i = meta.len() - 1;
          res += i;
          tree = &meta[i];
        }
        RadixTreeData::Leaves(leaves) => {
          res += leaves.len();
          break;
        }
      };

      res *= N;
    }

    res
  }

  pub fn push(&mut self, value: T) {
    let mut tree: &mut RadixTreeData<T, N> = Rc::make_mut(&mut self.0);

    loop {
      match tree {
        RadixTreeData::Meta(meta) => {
          let last = meta.len() - 1;
          tree = Rc::make_mut(&mut meta[last].0);
        }
        RadixTreeData::Leaves(leaves) => {
          if leaves.is_full() {
            break;
          }

          leaves.push(value);
          return;
        }
      }
    }

    let mut tree: &RadixTree<T, N> = self;
    let mut max_depth_with_space = 0;
    let mut depth = 1;

    loop {
      match &*tree.0 {
        RadixTreeData::Meta(meta) => {
          if !meta.is_full() {
            max_depth_with_space = depth;
          }

          let last = meta.len() - 1;
          tree = &meta[last];
        }
        RadixTreeData::Leaves(leaves) => {
          assert!(leaves.is_full());
          break;
        }
      }

      depth += 1;
    }

    if max_depth_with_space == 0 {
      let mut swap_node = Self::new_meta();
      swap(&mut swap_node, self);

      let self_meta = match Rc::make_mut(&mut self.0) {
        RadixTreeData::Meta(meta) => meta,
        RadixTreeData::Leaves(_) => {
          panic!("Should not happen because we just swapped meta into self")
        }
      };

      self_meta.push(swap_node);

      max_depth_with_space = 1;
      depth += 1;
    }

    let mut tree_with_space: &mut RadixTreeData<T, N> = Rc::make_mut(&mut self.0);

    for _ in 1..max_depth_with_space {
      match tree_with_space {
        RadixTreeData::Meta(meta) => {
          let last = meta.len() - 1;
          tree_with_space = Rc::make_mut(&mut meta[last].0);
        }
        RadixTreeData::Leaves(_leaves) => {
          panic!("Should have found meta with space");
        }
      }
    }

    let mut meta_node_with_space = match tree_with_space {
      RadixTreeData::Meta(meta) => meta,
      RadixTreeData::Leaves(_) => panic!("Should not happen"),
    };

    for _ in max_depth_with_space..(depth - 1) {
      let last = meta_node_with_space.len();
      meta_node_with_space.push(Self::new_meta());

      meta_node_with_space = match Rc::make_mut(&mut meta_node_with_space[last].0) {
        RadixTreeData::Meta(meta) => meta,
        RadixTreeData::Leaves(_) => panic!("Should not happen because we just pushed a meta node"),
      };
    }

    let mut new_leaves = ArrayVec::new();
    new_leaves.push(value);
    meta_node_with_space.push(RadixTree(Rc::new(RadixTreeData::Leaves(new_leaves))));
  }
}

impl<T: Clone, const N: usize> Default for RadixTree<T, N> {
  fn default() -> Self {
    Self::new()
  }
}
