export default function main() {
  let x = 0;
  x++; // Should fail compilation due to capture on line 6

  function foo() {
    return x;
  }

  return foo();
}
