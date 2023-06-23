//! test_output_slow(104743)

import Range from "../helpers/Range.ts";

export default function main() {
  return Range.primes().at(10_000);
}
