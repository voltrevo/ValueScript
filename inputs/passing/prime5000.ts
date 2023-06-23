//! bench()

import Range from "./helpers/Range.ts";

export default function main() {
  return Range.primes().at(4999);
}
