//! test_output_slow(104743)

import { Range_primes } from "../helpers/range.ts";

export default function main() {
  return Range_primes().at(10_000);
}
