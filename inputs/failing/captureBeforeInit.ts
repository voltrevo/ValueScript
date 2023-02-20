export default function main() {
  let res = foo(); // Should fail compilation: Binds uninitialized variable

  let x = 3;

  function foo() {
    return x;
  }

  return res;
}
