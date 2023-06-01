import { primes } from "./helpers/primes.ts";

export default function main() {
  let i = 1;

  for (const p of primes()) {
    if (i === 10000) {
      return p;
    }

    i++;
  }
}
