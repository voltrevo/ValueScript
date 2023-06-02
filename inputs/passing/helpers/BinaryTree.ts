import type NotNullish from "./NotNullish.ts";

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
      yield* this.left;
    }

    if (this.value) {
      yield this.value;
    }

    if (this.right) {
      yield* this.right;
    }
  }
}
