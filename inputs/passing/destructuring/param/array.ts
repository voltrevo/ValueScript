// test_output! 3

export default function main() {
  return foo([1, 2]);
}

function foo([a, b]: [number, number]) {
  return a + b;
}
