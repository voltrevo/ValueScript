import nil from './helpers/nil';

function blockTrim(text: string) {
  let lines = text.split('\n');

  while (lines.length > 0 && /^ *$/.test(lines[0])) {
    lines.shift();
  }

  while (lines.length > 0 && /^ *$/.test(lines[lines.length - 1])) {
    lines.pop();
  }

  let minIndent = Infinity;

  for (const line of lines) {
    if (line.trim() === '') {
      continue;
    }

    const match = line.match(/^ */);

    if (match === null || match[0].length >= minIndent) {
      continue;
    }

    minIndent = match[0].length;
  }

  lines = lines.map((line) => line.slice(minIndent));

  return lines.join('\n');
}

const files: Record<string, string | nil> = {
  'tutorial/hello.ts': blockTrim(`
    // Welcome to the ValueScript playground!
    //
    // This playground also acts as a tutorial by describing a variety of
    // examples. All examples are editable with live updates to their outputs.
    //
    // Keeping with tradition, here is the hello world program.

    export default function main() {
      return "Hello world!";
    }

    // When you're ready, click the next arrow ('>') above to continue.
  `),

  'tutorial/valueSemantics.ts': blockTrim(`
    export default function main() {
      const leftBowl = ['apple', 'mango'];

      let rightBowl = leftBowl;
      rightBowl.push('peach');

      return leftBowl.includes("peach");
      // TypeScript:  true
      // ValueScript: false
    }

    // In TypeScript, \`leftBowl\` and \`rightBowl\` are the same object, and
    // that object changes. In ValueScript, objects are just data, they don't
    // change. When you change \`rightBowl\`, you are changing the *variable*
    // and therefore \`leftBowl\` doesn't change.
  `),

  'tutorial/revertOnCatch.ts': blockTrim(`
    export default function () {
      let x = 0;

      try {
        x++;
        throw new Error("boom");
      } catch {}
    
      return x;
      // TypeScript:  1
      // ValueScript: 0
    }

    // In ValueScript, a try block is a transaction - it either runs to
    // completion, or it is reverted. This is impractical in TypeScript,
    // but in ValueScript all we have to do is snapshot the variables and
    // restore from them on catch. This works because all mutation is
    // local - nothing else can be affected.
  `),

  'examples/factorial.ts': blockTrim(`
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

  'examples/counter.ts': blockTrim(`
    export default function main() {
      let c = new Counter();

      return [c.get(), c.get(), c.get()];
    }

    class Counter {
      next = 1;

      get() {
        return this.next++;
      }
    }
  `),

  'examples/reverse.ts': blockTrim(`
    export default function main() {
      const values = ['a', 'b', 'c'];
    
      return [values, reverse(values)];
    }
    
    function reverse<T>(arr: T[]) {
      let left = 0;
      let right = arr.length - 1;
    
      while (left < right) {
        [arr[left], arr[right]] = [arr[right], arr[left]];
    
        left++;
        right--;
      }
    
      return arr;
    
      // This version also works:
      //   arr.reverse();
      //   return arr;
    }
  `),

  'examples/binaryTree.ts': blockTrim(`
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

  'examples/mergeSort.ts': blockTrim(`
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

  'examples/quickSort.ts': blockTrim(`
    export default function main() {
      const x = [7, 18, 9, 11, 16, 3, 8, 2, 5, 4, 6, 14, 15, 17, 10, 12, 1, 13];
    
      return quickSort(x, (a, b) => a - b);
    }
    
    function quickSort<T>(vals: T[], cmp: (a: T, b: T) => number) {
      // Demonstrates the ability to do in-place updates in ValueScript.
      //
      // There's only one reference to \`vals\`, so we can mutate it in-place
      // without violating value semantics.
      //
      // (At the time of writing the internals aren't very careful about
      // minimizing ref counters so this might not be happening, but the logic
      // to mutate in-place when the ref count is one is already there. This
      // will be optimized/fixed in the future.)
      //
      // More on quickSort: 
      // https://www.youtube.com/watch?v=Hoixgm4-P4M
    
      const len = vals.length;
      let ranges: [number, number][] = [[0, len - 1]];
    
      while (true) {
        const range = ranges.shift();
    
        if (!range) {
          return vals;
        }
    
        const [start, end] = range;
    
        if (end - start <= 0) {
          continue;
        }
    
        let i = start;
        let j = end;
    
        let pivotIndex = Math.floor((i + j) / 2);
        [vals[pivotIndex], vals[j]] = [vals[j], vals[pivotIndex]];
        const pivot = vals[j];
        j--;
    
        while (true) {
          while (cmp(vals[i], pivot) < 0) {
            i++;
          }
    
          while (cmp(vals[j], pivot) > 0) {
            j--;
          }
    
          if (i < j) {
            [vals[i], vals[j]] = [vals[j], vals[i]];
            i++;
            j--;
            continue;
          }
          
          [vals[i], vals[end]] = [vals[end], vals[i]];
    
          ranges.push([start, i - 1]);
          ranges.push([i + 1, end]);
    
          break;
        }
      }
    }  
  `),

  'examples/idGenerationError.ts': blockTrim(`
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

  'examples/idGeneration.ts': blockTrim(`
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

  'examples/sideEffectsArticle/enablePirateError.ts': blockTrim(`
    export default function main() {
      let pirateEnabled = false;

      function greet() {
        if (!pirateEnabled) {
          return "Hi";
        }
  
        return "Ahoy";
      }
  
      function enablePirate() {
        pirateEnabled = true;
        return "Done";
      }
  
      return [
        greet(),
        enablePirate(),
        greet(),
      ];
    }
  `),

  'examples/sideEffectsArticle/lyingAboutA.ts': blockTrim(`
    export default function main() {
      let a = 5;
      a += 2;
      
      return a;
    }
  `),

  'examples/sideEffectsArticle/add1To50WithMutation.ts': blockTrim(`
    export default function main() {
      let sum = 0;

      for (let i = 1; i <= 50; i++) {
        sum += i;
      }

      return sum;
    }
  `),

  'examples/sideEffectsArticle/add1To50WithoutMutation.ts': blockTrim(`
    export default function main() {
      return makeRange(1, 51)
        .reduce((a, b) => a + b);
    }
    
    function makeRange(start: number, end: number): number[] {
      if (start === end) {
        return [];
      }
    
      return [start].concat(
        makeRange(start + 1, end),
      );
    }
  `),

  'examples/sideEffectsArticle/enablePirateWorkaround.ts': blockTrim(`
    export default function main() {
      let pirateEnabled = false;

      function greet(pirateEnabled: boolean) {
        if (!pirateEnabled) {
          return "Hi";
        }
  
        return "Ahoy";
      }
  
      function enablePirate(
        pirateEnabled: boolean,
      ): [boolean, string] {
        pirateEnabled = true;
        return [pirateEnabled, "Done"];
      }

      const greetResponse1 = greet(pirateEnabled);

      let enablePirateResponse: string;
      [pirateEnabled, enablePirateResponse] = enablePirate(pirateEnabled);
  
      const greetResponse2 = greet(pirateEnabled);
  
      return [
        greetResponse1,
        enablePirateResponse,
        greetResponse2,
      ];
    }
  `),

  'examples/sideEffectsArticle/actorEnablePirate.ts': blockTrim(`
    export default function main() {
      let actor = new Actor();
  
      return [
        actor.greet(),
        actor.enablePirate(),
        actor.greet(),
      ];
    }

    class Actor {
      pirateEnabled = false;

      greet() {
        if (!this.pirateEnabled) {
          return "Hi";
        }
  
        return "Ahoy";
      }

      enablePirate() {
        this.pirateEnabled = true;
        return "Done";
      }
    }
  `),
};

export default files;
