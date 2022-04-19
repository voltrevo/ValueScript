function eg1() {
  let x = [];     // [] set $x
  x.push(x);      // get $x callMethod $x 'push' 1
  console.log(x); // get $x console callMethod 'log' 1
}

function eg2() {
  let x = [];   // [] set $x
  let y = [];   // [] set $y

  x.push('hi'); // 'hi' get $x 'push' at callMethod $x 1
}

function eg3() {
  class Foo {
    x = 0;

    inc() {
      this.x++;
    }
  }

  let f = new Foo();
  f.inc();
  console.log(f.x); // 1

  let g = f;
  g.inc();
  console.log(g.x); // 2
  console.log(f.x); // 1
}

function eg4() {
  let x = 0;

  function inc() {
    x++;
    return x;
  }

  console.log(inc()); // 1
  console.log(inc()); // 1
  console.log(inc()); // 1

  console.log(x); // 0
  x++;
  console.log(x); // 1
  x++;
  console.log(x); // 2

  console.log(inc()); // 1
}

function eg5() {
  class BinaryTree {
    constructor(x) {
      this.value = x;
    }

    insert(x) {
      if (this.value === undefined) {
        this.value = x;
        return;
      }

      if (x < this.value) {
        if (this.left === undefined) {
          this.left = new BinaryTree(x);
        } else {
          this.left.insert(x);
        }
      } else {
        if (this.right === undefined) {
          this.right = new BinaryTree(x);
        } else {
          this.right.insert(x);
        }
      }
    }

    contains(x) {
      if (this.value === undefined) {
        return false;
      }

      if (x < this.value) {
        return this.left !== undefined && this.left.contains(x);
      }

      return this.right !== undefined && this.right.contains(x);
    }
  }

  let tree = new BinaryTree();

  tree.insert(5);
  console.log(tree.contains(5)); // true
  console.log(tree.contains(7)); // false

  let tree2 = tree;
  tree2.insert(7);
  console.log(tree.contains(7)); // false
  console.log(tree2.contains(7)); // true
}

function eg7() {
  class Foo {
    bar() {
      this = 3;
    }
  }

  let f = new Foo();
  console.log(f); // Foo{}

  f.bar();

  console.log(f); // 3
}
