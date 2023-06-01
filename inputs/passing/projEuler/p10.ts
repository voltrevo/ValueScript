import { primes } from "./helpers/primes.ts";

export default function main() {
  let sum = 0;

  for (const p of primes()) {
    if (p >= 2000000) {
      return sum;
    }

    sum += p;
  }
}
