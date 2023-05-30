//! test_output(3)

export default function main() {
  const x = 3;

  function foo() {
    return bar();
  }

  function bar() {
    return x;
  }

  return foo();
}
