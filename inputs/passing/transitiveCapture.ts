export default function() {
  const x = 3;

  function foo() {
    return bar();
  }

  function bar() {
    return x;
  }

  return foo();
}
