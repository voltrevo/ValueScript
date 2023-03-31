export default function main() {
  function foo() {
    return x; // During tdz, but this is ok. Functions are hoisted anyway.
    //        // (Currently emits error incorrectly.)
  }

  const x = 3;

  // It's the references that matter. This is ok because x is initialized.
  return foo();
}
