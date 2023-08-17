use std::{
  mem::swap,
  ops::{Index, IndexMut},
  rc::Rc,
};

use arrayvec::ArrayVec;

use crate::strict_radix_tree_iterator::StrictRadixTreeIterator;

#[derive(Clone)]
pub(crate) enum StrictRadixTreeData<T, const N: usize> {
  Meta(ArrayVec<StrictRadixTree<T, N>, N>),
  Leaves(ArrayVec<T, N>),
}

#[derive(Clone)]
pub struct StrictRadixTree<T, const N: usize>(Rc<StrictRadixTreeData<T, N>>);

impl<T: Clone, const N: usize> StrictRadixTree<T, N> {
  pub fn new() -> Self {
    StrictRadixTree::<T, N>(Rc::new(
      StrictRadixTreeData::<T, N>::Leaves(ArrayVec::new()),
    ))
  }

  pub fn clear(&mut self) {
    match self.data_mut() {
      StrictRadixTreeData::Leaves(leaves) => leaves.clear(),
      data_mut => {
        *data_mut = StrictRadixTreeData::Leaves(ArrayVec::new());
      }
    }
  }

  pub fn is_empty(&self) -> bool {
    match self.data() {
      StrictRadixTreeData::Meta(_) => false,
      StrictRadixTreeData::Leaves(leaves) => leaves.is_empty(),
    }
  }

  pub fn len(&self) -> usize {
    let mut res = 0;
    let mut tree = self;

    loop {
      match tree.data() {
        StrictRadixTreeData::Meta(meta) => {
          let i = meta.len() - 1;
          res += i;
          tree = &meta[i];
        }
        StrictRadixTreeData::Leaves(leaves) => {
          res += leaves.len();
          break;
        }
      };

      res *= N;
    }

    res
  }

  pub fn push(&mut self, value: T) {
    let mut tree: &mut StrictRadixTreeData<T, N> = Rc::make_mut(&mut self.0);

    loop {
      match tree {
        StrictRadixTreeData::Meta(meta) => {
          let last = meta.len() - 1;
          tree = Rc::make_mut(&mut meta[last].0);
        }
        StrictRadixTreeData::Leaves(leaves) => {
          if leaves.is_full() {
            break;
          }

          leaves.push(value);
          return;
        }
      }
    }

    let mut tree: &StrictRadixTree<T, N> = self;
    let mut max_depth_with_space = 0;
    let mut depth = 1;

    loop {
      match tree.data() {
        StrictRadixTreeData::Meta(meta) => {
          if !meta.is_full() {
            max_depth_with_space = depth;
          }

          let last = meta.len() - 1;
          tree = &meta[last];
        }
        StrictRadixTreeData::Leaves(leaves) => {
          assert!(leaves.is_full());
          break;
        }
      }

      depth += 1;
    }

    if max_depth_with_space == 0 {
      let mut swap_node = Self::new_meta();
      swap(&mut swap_node, self);

      let self_meta = match self.data_mut() {
        StrictRadixTreeData::Meta(meta) => meta,
        StrictRadixTreeData::Leaves(_) => {
          panic!("Should not happen because we just swapped meta into self")
        }
      };

      self_meta.push(swap_node);

      max_depth_with_space = 1;
      depth += 1;
    }

    let mut tree_with_space: &mut StrictRadixTreeData<T, N> = Rc::make_mut(&mut self.0);

    for _ in 1..max_depth_with_space {
      match tree_with_space {
        StrictRadixTreeData::Meta(meta) => {
          let last = meta.len() - 1;
          tree_with_space = Rc::make_mut(&mut meta[last].0);
        }
        StrictRadixTreeData::Leaves(_leaves) => {
          panic!("Should have found meta with space");
        }
      }
    }

    let mut meta_node_with_space = match tree_with_space {
      StrictRadixTreeData::Meta(meta) => meta,
      StrictRadixTreeData::Leaves(_) => panic!("Should not happen"),
    };

    for _ in max_depth_with_space..(depth - 1) {
      let last = meta_node_with_space.len();
      meta_node_with_space.push(Self::new_meta());

      meta_node_with_space = match Rc::make_mut(&mut meta_node_with_space[last].0) {
        StrictRadixTreeData::Meta(meta) => meta,
        StrictRadixTreeData::Leaves(_) => {
          panic!("Should not happen because we just pushed a meta node")
        }
      };
    }

    let mut new_leaves = ArrayVec::new();
    new_leaves.push(value);
    meta_node_with_space.push(StrictRadixTree(Rc::new(StrictRadixTreeData::Leaves(
      new_leaves,
    ))));
  }

  pub fn pop(&mut self) -> Option<T> {
    let mut tree = &mut *self;

    loop {
      match tree.data_mut() {
        StrictRadixTreeData::Meta(meta) => {
          let last = meta.len() - 1;
          tree = &mut meta[last];
        }
        StrictRadixTreeData::Leaves(leaves) => {
          let res = leaves.pop();

          if leaves.is_empty() {
            self.truncate(self.len());
          }

          return res;
        }
      }
    }
  }

  pub fn truncate(&mut self, len: usize) {
    if len == 0 {
      self.clear();
      return;
    }

    let path = match self.index_path(len - 1) {
      Some(path) => path,
      None => return,
    };

    let mut tree = &mut *self;

    for p in path {
      match tree.data_mut() {
        StrictRadixTreeData::Meta(meta) => {
          if meta.len() <= p {
            break;
          }

          meta.truncate(p + 1);
          tree = &mut meta[p];
        }
        StrictRadixTreeData::Leaves(leaves) => {
          leaves.truncate(p + 1);
          break;
        }
      }
    }

    while let StrictRadixTreeData::Meta(meta) = self.data_mut() {
      if meta.len() > 1 {
        break;
      }

      *self = meta.pop().unwrap();
    }
  }

  pub fn first(&self) -> Option<&T> {
    let mut tree = self;

    loop {
      match tree.data() {
        StrictRadixTreeData::Meta(meta) => {
          tree = &meta[0];
        }
        StrictRadixTreeData::Leaves(leaves) => break leaves.first(),
      }
    }
  }

  pub fn first_mut(&mut self) -> Option<&mut T> {
    let mut tree = self;

    loop {
      match tree.data_mut() {
        StrictRadixTreeData::Meta(meta) => {
          tree = &mut meta[0];
        }
        StrictRadixTreeData::Leaves(leaves) => break leaves.first_mut(),
      }
    }
  }

  pub fn last(&self) -> Option<&T> {
    let mut tree = self;

    loop {
      match tree.data() {
        StrictRadixTreeData::Meta(meta) => {
          let last = meta.len() - 1;
          tree = &meta[last];
        }
        StrictRadixTreeData::Leaves(leaves) => break leaves.last(),
      }
    }
  }

  pub fn last_mut(&mut self) -> Option<&mut T> {
    let mut tree = self;

    loop {
      match tree.data_mut() {
        StrictRadixTreeData::Meta(meta) => {
          let last = meta.len() - 1;
          tree = &mut meta[last];
        }
        StrictRadixTreeData::Leaves(leaves) => break leaves.last_mut(),
      }
    }
  }

  pub fn get(&self, i: usize) -> Option<&T> {
    let mut tree = self;

    for p in tree.index_path(i)? {
      match tree.data() {
        StrictRadixTreeData::Meta(meta) => {
          tree = meta.get(p)?;
        }
        StrictRadixTreeData::Leaves(leaves) => {
          return leaves.get(p);
        }
      }
    }

    None
  }

  pub fn get_mut(&mut self, i: usize) -> Option<&mut T> {
    let mut tree = self;

    for p in tree.index_path(i)? {
      match tree.data_mut() {
        StrictRadixTreeData::Meta(meta) => {
          tree = meta.get_mut(p)?;
        }
        StrictRadixTreeData::Leaves(leaves) => {
          return leaves.get_mut(p);
        }
      }
    }

    None
  }

  fn new_meta() -> Self {
    StrictRadixTree::<T, N>(Rc::new(StrictRadixTreeData::<T, N>::Meta(ArrayVec::new())))
  }

  pub(crate) fn data(&self) -> &StrictRadixTreeData<T, N> {
    &self.0
  }

  fn data_mut(&mut self) -> &mut StrictRadixTreeData<T, N> {
    Rc::make_mut(&mut self.0)
  }

  pub(crate) fn depth(&self) -> usize {
    let mut res = 1;
    let mut tree = self;

    while let StrictRadixTreeData::Meta(meta) = tree.data() {
      tree = &meta[0];
      res += 1;
    }

    res
  }

  fn index_path(&self, mut i: usize) -> Option<Vec<usize>> {
    let mut path = vec![0; self.depth()];

    for p in path.iter_mut().rev() {
      *p = i % N;
      i /= N;
    }

    match i {
      0 => Some(path),
      _ => None,
    }
  }
}

impl<T: Clone, const N: usize> Default for StrictRadixTree<T, N> {
  fn default() -> Self {
    Self::new()
  }
}

impl<T: Clone, const N: usize> Index<usize> for StrictRadixTree<T, N> {
  type Output = T;

  fn index(&self, i: usize) -> &T {
    let mut tree = self;

    for p in tree.index_path(i).expect("Out of bounds") {
      match tree.data() {
        StrictRadixTreeData::Meta(meta) => {
          tree = &meta[p];
        }
        StrictRadixTreeData::Leaves(leaves) => {
          return &leaves[p];
        }
      }
    }

    panic!("Out of bounds");
  }
}

impl<T: Clone, const N: usize> IndexMut<usize> for StrictRadixTree<T, N> {
  fn index_mut(&mut self, i: usize) -> &mut T {
    let mut tree = self;

    for p in tree.index_path(i).expect("Out of bounds") {
      match tree.data_mut() {
        StrictRadixTreeData::Meta(meta) => {
          tree = &mut meta[p];
        }
        StrictRadixTreeData::Leaves(leaves) => {
          return &mut leaves[p];
        }
      }
    }

    panic!("Out of bounds");
  }
}

impl<'a, T: Clone, const N: usize> IntoIterator for &'a StrictRadixTree<T, N> {
  type Item = &'a T;
  type IntoIter = StrictRadixTreeIterator<'a, T, N>;

  fn into_iter(self) -> Self::IntoIter {
    StrictRadixTreeIterator::new(self)
  }
}
