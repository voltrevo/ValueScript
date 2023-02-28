export default function main(n: string) {
  return factorial(+n);
}

function factorial(n: number): number {
  if (n === 0) {
    return 1;
  }

  return n * factorial(n - 1);
}
