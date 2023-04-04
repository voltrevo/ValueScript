// test_output! 1
// (This is wrong.)

export default function () {
  const foo = new Foo();
  foo.callInc(); // Should throw

  return foo.x;
}

class Foo {
  x = 0;

  inc() {
    this.x++;
  }

  callInc() {
    this.inc(); // Needs to propagate the constness
  }
}
