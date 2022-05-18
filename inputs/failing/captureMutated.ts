export default function() {
  let x = 0;
  x++;

  function foo() {
    return x; // Should fail compilation due to capture on line 3
  }

  return foo();
}
