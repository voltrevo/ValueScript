// test_output! E: TypeError{"message":"Cannot mutate this because it is const"}

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
