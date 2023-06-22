//! test_output_slow(-59231)

import { isPrime } from "./helpers/primes.ts";

export default function main() {
  let best = {
    a: 0,
    b: 0,
    n: 0,
  };

  for (let a = -999; a < 1000; a++) {
    for (let b = -999; b < 1000; b++) {
      let n = 0;

      while (true) {
        const p = n * n + a * n + b;

        if (p < 2) {
          break;
        }

        if (!isPrime(p)) {
          break;
        }

        n++;
      }

      if (n > best.n) {
        best = { a, b, n };
      }
    }
  }

  return best.a * best.b;
}
