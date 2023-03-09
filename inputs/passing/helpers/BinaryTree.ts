export default class BinaryTree<T> {
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

    if (this.left) {
      res = res.concat(this.left.toArray());
    }

    if (this.value !== undefined) {
      res.push(this.value);
    }

    if (this.right) {
      res = res.concat(this.right.toArray());
    }

    return res;
  }
}
