//! test_output(E: TypeError{"message":"Cannot mutate this because it is const"})

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
