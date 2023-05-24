export default function main() {
  return factorial(5);
}

function factorial(n: number): number {
  if (n === 0) {
    return 1;
  }

  return n * factorial(n - 1);
}
