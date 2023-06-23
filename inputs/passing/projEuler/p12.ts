//! test_output_slow(76576500)

import Range from "../helpers/Range.ts";
import { factorizeAsPowers } from "./helpers/primes.ts";

export default function main() {
  return Range.from(triangularNumbers())
    .filter((tri) => countFactors(tri) > 500)
    .first();
}

function countFactors(n: number): number {
  return Range.from(factorizeAsPowers(n))
    .map(([_, power]) => power + 1)
    .product();
}

function* triangularNumbers() {
  let sum = 0;

  for (let i = 1;; i++) {
    sum += i;
    yield sum;
  }
}
