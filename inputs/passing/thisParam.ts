//! test_output(10)

export default function () {
  return foo(5);
}

function foo(this: unknown, x: number) {
  return 2 * x;
}
