//! bench()

export default function main() {
  return fib(25);
}

function fib(n: number): number {
  if (n < 2) {
    return n;
  }

  return fib(n - 1) + fib(n - 2);
}
