//! test_output(37)

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
