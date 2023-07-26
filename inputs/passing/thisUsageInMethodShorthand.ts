//! test_output(["baz","baz"])

export default function main() {
  let stuff = {
    foo() {
      this.bar = "baz";
      return this.bar;
    },
    bar: "bar",
  };

  return [
    stuff.foo(),
    stuff.bar,
  ];
}
