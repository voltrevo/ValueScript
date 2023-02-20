export default function main() {
  function foo() {
    const x = 3;
    return x + bar();
  }

  const x = 4;

  function bar() {
    return x;
  }

  return foo();
}
