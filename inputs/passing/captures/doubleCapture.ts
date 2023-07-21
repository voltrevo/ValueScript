//! test_output(37)

export default function main() {
  const x = 37;

  function foo() {
    return () => x;
  }

  return foo()();
}
