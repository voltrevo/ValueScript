//! test_output(0)

export default function main() {
  let foo = new Foo();
  foo.inc();

  return foo.x;
}

class Foo {
  x = 0;

  inc() {
    try {
      this.x++;
      throw new Error("boom");
    } catch {}
  }
}
