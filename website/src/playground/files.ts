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

  "tutorial/factorial.ts": blockTrim(`
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

  "tutorial/binaryTree.ts": blockTrim(`
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
  `),
};

export default files;
