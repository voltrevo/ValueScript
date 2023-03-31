export default function main() {
  const foo = () => bar(); // Should fail compilation: Binds uninitialized variable

  const x = 3;

  function bar() {
    return x;
  }

  return foo();
}
