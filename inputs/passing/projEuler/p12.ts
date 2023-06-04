import range from "../helpers/range.ts";
import { factorizeAsPowers } from "./helpers/primes.ts";

export default function main() {
  return range(triangularNumbers())
    .filter(tri => countFactors(tri) > 500)
    .first();
}

function countFactors(n: number): number {
  return range(factorizeAsPowers(n))
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
