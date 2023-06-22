//! test_output_slow(142913828922)

import { Range_primes } from "../helpers/range.ts";

export default function main() {
  return Range_primes()
    .while((p) => p < 2_000_000)
    .sum();
}
