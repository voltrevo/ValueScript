//! test_output("matched 42")

export default function foo() {
  switch (echo(21) + 21) {
    case 1:
      return "matched 1";
    case echo(40) + 2:
      return "matched 42";
    default:
      return "default";
  }
}

function echo<T>(x: T) {
  return x;
}
