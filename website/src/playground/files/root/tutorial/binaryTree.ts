// Value semantics have a deep effect on how your code works. Here we have a
// binary tree that follows the rules of local-only mutation, but if you look
// inside the implementation you'll see it's written in the most straightforward
// way, without any regard to following any functional rules.
//
// This is because the value semantics of ValueScript are guaranteed by the
// language itself. In JavaScript you'd need to carefully follow the functional
// rules, but in ValueScript it's free.

import BinaryTree from "../lib/BinaryTree";

export default function main() {
  let tree = new BinaryTree<number>();

  tree.insert(2);
  tree.insert(5);
  tree.insert(1);

  const treeSnapshot = tree;

  tree.insert(3);
  tree.insert(4);

  return [treeSnapshot.toArray(), tree.toArray()];
  // JavaScript:  [[1, 2, 3, 4, 5], [1, 2, 3, 4, 5]]
  // ValueScript: [[1, 2, 5], [1, 2, 3, 4, 5]]
}

// This is why ValueScript needs its own runtime. It's not a thin wrapper around
// JavaScript or just a linting rule. As a bonus, ValueScript can also run
// completely independently of a JavaScript environment.
