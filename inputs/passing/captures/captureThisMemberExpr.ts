//! test_output("bar")

export default function () {
  const foo = new Foo();
  const barCaller = foo.barCaller();

  return barCaller();
}

class Foo {
  barString = "bar";

  bar() {
    return this.barString;
  }

  barCaller() {
    return () => this.bar();
  }
}
