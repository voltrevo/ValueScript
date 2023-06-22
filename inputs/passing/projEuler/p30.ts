//! test_output_slow(443839)

import range, { Range_numbers } from "../helpers/range.ts";

export default function main() {
  return Range_numbers(10, 1_000_000)
    .flatMap(function* (i) {
      const digitsPowSum = range(`${i}`)
        .map((d) => Number(d) ** 5)
        .sum();

      if (i === digitsPowSum) {
        yield i;
      }
    })
    .sum();
}
