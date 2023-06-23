//! test_output_slow(142913828922)

import Range from "../helpers/Range.ts";

export default function main() {
  return Range.primes()
    .while((p) => p < 2_000_000)
    .sum();
}
