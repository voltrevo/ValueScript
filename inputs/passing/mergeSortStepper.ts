// test_output((TODO broken!) [1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18])

declare const Debug: {
  log: (...args: unknown[]) => undefined;
};

export default function main() {
  let stepper = new MergeSortStepper(
    [7, 18, 9, 11, 16, 3, 8, 2, 5, 4, 6, 14, 15, 17, 10, 12, 1, 13],
    (a, b) => {
      let swap = a > b;
      Debug.log({ a, b, swap });
      return a - b;
    },
  );

  let count = 0;

  while (true) {
    if (stepper.step()) {
      count++;
    } else {
      break;
    }
  }

  return {
    count,
    vals: (stepper.tree.data as any).vals,
  };
}

class MergeSortStepper<T> {
  tree: MergeSortNode<T>
  cmp: (a: T, b: T) => number;

  constructor(vals: T[], cmp: (a: T, b: T) => number) {
    this.tree = makeTree(vals);
    this.cmp = cmp;
  }

  step(): boolean {
    return this.tree.step(this.cmp);
  }
}

class MergeSortNode<T> {
  data: MergeSortNodeData<T>;

  constructor(data: MergeSortNodeData<T>) {
    this.data = data;
  }

  step(cmp: (a: T, b: T) => number): boolean {
    if (this.data.type === 'sorted') {
      return false;
    }

    if (this.data.type === 'sorting') {
      if (this.data.left.length === 0 || this.data.right.length === 0) {
        let vals = this.data.vals;
        vals = vals.concat(this.data.left);
        vals = vals.concat(this.data.right);

        this.data = {
          type: 'sorted',
          vals,
        };

        return false;
      }

      const ordered = cmp(this.data.left[0], this.data.right[0]) <= 0;
      let selection: T;

      if (ordered) {
        selection = this.data.left.shift()!;
      } else {
        selection = this.data.right.shift()!;
      }

      this.data.vals.push(selection);

      return true;
    }

    if (this.data.left.step(cmp)) {
      return true;
    }

    if (this.data.right.step(cmp)) {
      return true;
    }

    assert(this.data.left.data.type === 'sorted');
    assert(this.data.right.data.type === 'sorted');

    this.data = {
      type: 'sorting',
      vals: [],
      left: this.data.left.data.vals,
      right: this.data.right.data.vals,
    };

    return this.step(cmp);
  }
}

type MergeSortNodeData<T> = (
  | { type: 'tree', left: MergeSortNode<T>, right: MergeSortNode<T> }
  | { type: 'sorting', vals: T[], left: T[], right: T[] }
  | { type: 'sorted', vals: T[] }
);

function makeTree<T>(vals: T[]): MergeSortNode<T> {
  if (vals.length <= 1) {
    return new MergeSortNode({ type: 'sorted', vals });
  }

  if (vals.length === 2) {
    return new MergeSortNode({
      type: 'sorting',
      vals: [],
      left: [vals[0]],
      right: [vals[1]],
    });
  }

  const mid = Math.floor(vals.length / 2);

  return new MergeSortNode({
    type: 'tree',
    left: makeTree(vals.slice(0, mid)),
    right: makeTree(vals.slice(mid)),
  });
}

function assert(value: boolean): asserts value {
  if (!value) {
    (undefined as any).boom; // TODO: Implement exceptions
  }
}
