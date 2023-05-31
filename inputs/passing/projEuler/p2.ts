//! test_output(4613732)

export default function main() {
  let sum = 0;

  let [fibLast, fib] = [0, 1];

  while (true) {
    [fibLast, fib] = [fib, fibLast + fib];

    if (fib > 4000000) {
      return sum;
    }

    if (fib % 2 === 0) {
      sum += fib;
    }
  }
}
