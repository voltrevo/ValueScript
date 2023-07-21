//! test_output("bar")

export default function () {
  const foo = new Foo();
  const cloner = foo.cloner();

  return cloner().bar();
}

class Foo {
  barString = "bar";

  bar() {
    return this.barString;
  }

  cloner() {
    return () => this;
  }
}
