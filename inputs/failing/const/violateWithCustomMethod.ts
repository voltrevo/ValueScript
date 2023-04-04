// test_output! 1
// (This is wrong.)

export default function () {
  const foo = new Foo();
  foo.inc(); // Should throw

  return foo.x;
}

class Foo {
  x = 0;

  inc() {
    this.x++;
  }
}
