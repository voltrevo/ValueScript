// //! test_output(["bar","baz","bar","baz"])

export default function main() {
  const bar = echo("bar");
  const baz = echo("baz");

  class Foo {
    x = bar;
    static y = () => baz;
    bar() {
      return bar;
    }
    baz() {
      return baz;
    }
  }

  return [
    new Foo().x,
    Foo.y(),
    new Foo().bar(),
    new Foo().baz(),
  ];
}

function echo<T>(x: T) {
  return x;
}
