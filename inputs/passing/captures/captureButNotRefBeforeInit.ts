//! test_output(3)

export default function main() {
  function foo() {
    return x; // During tdz, but this is ok. Functions are hoisted anyway.
  }

  const x = 3;

  // It's the references that matter. This is ok because x is initialized.
  return foo();
}
