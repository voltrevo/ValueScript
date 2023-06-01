import type { NotNullish } from "./util.ts";

export default class BinaryTree<T extends NotNullish> {
  left?: BinaryTree<T>;
  value?: T;
  right?: BinaryTree<T>;

  insert(newValue: T) {
    if (this.value === undefined) {
      this.value = newValue;
      return;
    }

    if (newValue < this.value) {
      this.left ??= new BinaryTree();
      this.left.insert(newValue);
    } else {
      this.right ??= new BinaryTree();
      this.right.insert(newValue);
    }
  }

  *[Symbol.iterator](): Generator<T> {
    if (this.left) {
      for (const value of this.left) {
        yield value;
      }
    }

    if (this.value) {
      yield this.value;
    }

    if (this.right) {
      for (const value of this.right) {
        yield value;
      }
    }
  }
}
