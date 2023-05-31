//! test_output(120)

export default function main() {
  const one = 1;

  function factorial(n: number): number {
    if (n === 0) {
      return 1;
    }

    return n * factorial(n - one);
  }

  return factorial(5);
}
