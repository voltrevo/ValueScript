export default function main() {
  let x = 0;

  function foo() {
    x++; // Should fail compilation: mutates captures variable
    return x;
  }

  return foo();
}
