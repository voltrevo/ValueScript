export default function main() {
  const res = foo(); // Should fail compilation: Binds uninitialized variable
  const x = 3;

  function foo() {
    return x;
  }

  return res;
}
