import { nextOddPrime } from "./helpers/primes.ts";

export default function main() {
  let i = 1;
  let p = 2;

  while (i < 10001) {
    p = nextOddPrime(p);
    i++;
  }

  return p;
}
