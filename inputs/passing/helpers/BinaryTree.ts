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

  toArray() {
    let res: T[] = [];

    for (const value of this) {
      res.push(value);
    }

    return res;
  }

  [Symbol.iterator]() {
    let iter = new BinaryTreeIterator<T>();
    iter.stack.push({ type: "tree", data: this });

    return iter;
  }
}

class BinaryTreeIterator<T extends NotNullish> {
  stack:
    ({ type: "tree"; data: BinaryTree<T> } | { type: "value"; data: T })[] = [];

  next(): { done: true } | { value: T; done: false } {
    const item = this.stack.pop();

    if (item === undefined) {
      return { done: true };
    }

    if (item.type === "tree") {
      this.pushTree(item.data.right);
      this.pushValue(item.data.value);
      this.pushTree(item.data.left);

      return this.next();
    }

    return { value: item.data, done: false };
  }

  pushTree(tree?: BinaryTree<T>) {
    if (tree !== undefined) {
      this.stack.push({ type: "tree", data: tree });
    }
  }

  pushValue(value?: T) {
    if (value !== undefined) {
      this.stack.push({ type: "value", data: value });
    }
  }
}
