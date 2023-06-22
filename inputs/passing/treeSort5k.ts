//! bench()

import BinaryTree from "./helpers/BinaryTree.ts";
import randish from "./helpers/randish.ts";
import range from "./helpers/range.ts";

export default function main() {
  let tree = new BinaryTree<number>();

  for (const rand of range(randish()).limit(5_000)) {
    tree.insert(Math.floor(4_000 * rand));
  }

  return [...tree];
}
