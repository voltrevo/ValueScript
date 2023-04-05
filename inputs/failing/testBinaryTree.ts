// test_output! E: TypeError{"message":"Cannot mutate this because it is const"}
// (This is wrong.)

import BinaryTree from "./helpers/BinaryTree.ts";

export default function main() {
  let tree = new BinaryTree<number>();

  tree.insert(2);
  tree.insert(5);
  tree.insert(1);

  const treeSnapshot = tree;

  tree.insert(3);
  tree.insert(4);

  return [treeSnapshot.toArray(), tree.toArray()];
}
