// test_output! 3

export default function main() {
  return foo({ a: 1, b: 2 });
}

function foo({ a, b }: { a: number, b: number }) {
  return a + b;
}
