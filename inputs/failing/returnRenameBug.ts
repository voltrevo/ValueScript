//! test_output(0)
// Should be: undefined

export default function () {
  const x = foo();

  if (false) {
    return x;
  }
}

function foo() {
  return 0;
}
