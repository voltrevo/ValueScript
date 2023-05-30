//! test_output([[1,2,5],[1,2,3,4,5]])

export default function main() {
  let tree = BinaryTree();

  tree.insert(2);
  tree.insert(5);
  tree.insert(1);

  const treeSnapshot = tree;

  tree.insert(3);
  tree.insert(4);

  return [treeSnapshot.toArray(), tree.toArray()];
}

function BinaryTree() {
  type BinaryTree = {
    data: {
      value?: number,
      left?: BinaryTree,
      right?: BinaryTree,
    },
    insert(this: BinaryTree, newValue: number): void,
    toArray(this: BinaryTree): number[],
  };

  let tree: BinaryTree = {
    data: {},
    insert: function(newValue) {
      if (this.data.value === undefined) {
        this.data.value = newValue;
        return;
      }

      if (newValue < this.data.value) {
        this.data.left ??= BinaryTree();
        this.data.left.insert(newValue);
      } else {
        this.data.right ??= BinaryTree();
        this.data.right.insert(newValue);
      }
    },
    toArray: function() {
      let res: number[] = [];

      if (this.data.left) {
        res = cat(res, this.data.left.toArray());
      }

      if (this.data.value !== undefined) {
        res.push(this.data.value);
      }

      if (this.data.right) {
        res = cat(res, this.data.right.toArray());
      }

      return res;
    },
  };

  return tree;
}

function cat(left: number[], right: number[]) {
  for (let i = 0; i < right.length; i++) {
    left.push(right[i]);
  }

  return left;
}
