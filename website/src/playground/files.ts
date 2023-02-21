import nil from "./helpers/nil.ts";

function blockTrim(text: string) {
  let lines = text.split("\n");

  while (lines.length > 0 && /^ *$/.test(lines[0])) {
    lines.shift();
  }

  while (lines.length > 0 && /^ *$/.test(lines[lines.length - 1])) {
    lines.pop();
  }

  let minIndent = Infinity;

  for (const line of lines) {
    if (line.trim() === "") {
      continue;
    }

    const match = line.match(/^ */);

    if (match === null || match[0].length >= minIndent) {
      continue;
    }

    minIndent = match[0].length;
  }

  lines = lines.map((line) => line.slice(minIndent));

  return lines.join("\n");
}

const files: Record<string, string | nil> = {
  "tutorial/hello.ts": blockTrim(`
    // Welcome to the ValueScript playground!
    //
    // This playground also acts as a tutorial by describing a variety of
    // examples. Please go ahead and make edits to the code, you should see
    // the results in real-time!
    //
    // Keeping with tradition, here is the hello world program.

    export default function main() {
      return "Hello world!";
    }

    // When you're ready, click the next arrow ('>') above to continue.
  `),

  "tutorial/valueSemantics.ts": blockTrim(`
    export default function main() {
      const leftBowl = ['apple', 'mango'];

      let rightBowl = leftBowl;
      rightBowl.push('peach');

      return {
        leftBowl,
        rightBowl,
      };
    }

    // In TypeScript, leftBowl also contains 'peach':
    //
    // {
    //   leftBowl: ['apple', 'mango', 'peach'],
    //   rightBowl: ['apple', 'mango', 'peach'],
    // }
    //
    // This is because TypeScript interprets the code to mean that leftBowl and
    // rightBowl are the same object, and that object changes.
    //
    // In ValueScript, objects do not change, but variables do. Pushing onto
    // rightBowl is interpreted as a change to the rightBowl variable itself,
    // not the data it points to. rightBowl points to some new data, which may
    // reference the old data, but only as a performance optimization.
  `),

  "examples/factorial.ts": blockTrim(`
    export default function main() {
      return factorial(5);
    }

    function factorial(n: number): number {
      if (n === 0) {
        return 1;
      }

      return n * factorial(n - 1);
    }
  `),

  "examples/counter.ts": blockTrim(`
    export default function main() {
      let c = new Counter();

      return [c.get(), c.get(), c.get()];
    }

    class Counter {
      next: number;

      constructor() {
        this.next = 1;
      }

      get() {
        return this.next++;
      }
    }
  `),

  "examples/binaryTree.ts": blockTrim(`
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

    class BinaryTree<T> {
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
  `),

  "examples/mergeSort.ts": blockTrim(`
    export default function main() {
      const x = [7, 18, 9, 11, 16, 3, 8, 2, 5, 4, 6, 14, 15, 17, 10, 12, 1, 13];

      return mergeSort(x, (a, b) => a - b);
    }

    function mergeSort<T>(vals: T[], cmp: (a: T, b: T) => number): T[] {
      const len = vals.length;

      if (len <= 1) {
        return vals;
      }

      if (len === 2) {
        if (cmp(vals[0], vals[1]) > 0) {
          return [vals[1], vals[0]];
        }

        return vals;
      }

      const mid = vals.length / 2;

      const leftSorted = mergeSort(vals.slice(0, mid), cmp);
      const rightSorted = mergeSort(vals.slice(mid), cmp);

      let res: T[] = [];

      let left = 0;
      const leftLen = leftSorted.length;
      let right = 0;
      const rightLen = rightSorted.length;

      while (left < leftLen && right < rightLen) {
        if (cmp(leftSorted[left], rightSorted[right]) <= 0) {
          res.push(leftSorted[left++]);
        } else {
          res.push(rightSorted[right++]);
        }
      }

      while (left < leftLen) {
        res.push(leftSorted[left++]);
      }

      while (right < rightLen) {
        res.push(rightSorted[right++]);
      }

      return res;
    }
  `),

  "examples/idGenerationError.ts": blockTrim(`
    export default function main() {
      let nextId = 1;
          
      function generateId() {
        const result = nextId;
        nextId++;
      
        return result;
      }
  
      return [
        generateId(),
        generateId(),
        generateId(),
      ];
    }
  `),

  "examples/idGeneration.ts": blockTrim(`
    export default function main() {
      let idGen = new IdGenerator();
    
      return [
        idGen.generate(),
        idGen.generate(),
        idGen.generate(),
      ];
    }
    
    class IdGenerator {
      nextId: number;
    
      constructor() {
        this.nextId = 1;
      }
    
      generate() {
        const result = this.nextId;
        this.nextId++;
    
        return result;
      }
    }
  `),
};

export default files;
