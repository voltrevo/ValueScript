// test_output! E: TypeError{"message":"Cannot mutate this because it is const"}
// (This is wrong.)

export default function () {
  const foo = new Foo();
  const x = foo.calc();

  return x;
}

class Foo {
  calc() {
    return this.get();
  }

  get() {
    return 37;
  }
}
