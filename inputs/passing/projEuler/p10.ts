import { PrimeGenerator } from "./helpers/primes.ts";

export default function main() {
  let sum = 0;
  let gen = new PrimeGenerator();

  while (true) {
    const p = gen.next();

    if (p >= 2000000) {
      return sum;
    }

    sum += p;
  }
}
