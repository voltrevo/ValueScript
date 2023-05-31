export default function main() {
  const res = foo(); // Error: binds x before its declaration
  const x = 3;

  function foo() {
    return x;
  }

  return res;
}
