//! test_output_slow(443839)

import Range from "../helpers/Range.ts";

export default function main() {
  return Range.numbers(10, 1_000_000)
    .flatMap(function* (i) {
      const digitsPowSum = Range.from(`${i}`)
        .map((d) => Number(d) ** 5)
        .sum();

      if (i === digitsPowSum) {
        yield i;
      }
    })
    .sum();
}
