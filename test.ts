import { lessThan, staticAssert } from 'value-script';

function test(fn: () => void) {
  fn();
}

staticAssert(1 + 1 === 2);

test(() => {
  let x = [];
  x.push(x);

  staticAssert(x === [[]]);  
});

class BinaryTree<T> {
  value?: T;
  left?: BinaryTree<T>;
  right?: BinaryTree<T>;

  insert(newValue: T) {
    if (this.value === undefined) {
      this.value = newValue;
    } else if (lessThan(newValue, this.value)) {
      this.left ??= new BinaryTree<T>();
      this.left.insert(newValue);
    } else {
      this.right ??= new BinaryTree<T>();
      this.right.insert(newValue);
    }
  }

  contains(testValue: T): boolean {
    if (this.value === testValue) {
      return true;
    }

    if (lessThan(testValue, this.value)) {
      return this.left?.contains(this.value) ?? false;
    }

    return this.right?.contains(this.value) ?? false;
  }
}

test(() => {
  let tree = new BinaryTree<number>();

  tree.insert(1);
  tree.insert(2);
  tree.insert(3);

  staticAssert(tree.contains(1));
  staticAssert(tree.contains(2));
  staticAssert(tree.contains(3));

  let tree2 = tree;
  staticAssert(tree2 === tree);
  tree2.insert(4);
  staticAssert(tree2 !== tree);

  staticAssert(tree2.contains(1));
  staticAssert(tree2.contains(2));
  staticAssert(tree2.contains(3));
  staticAssert(tree2.contains(4));

  staticAssert(!tree.contains(4));
});
