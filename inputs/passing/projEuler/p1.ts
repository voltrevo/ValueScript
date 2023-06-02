//! test_output(233168)

import { Range_numbers } from "../helpers/range.ts";

export default function main() {
  return Range_numbers(0, 1000)
    .filter((x) => x % 3 === 0 || x % 5 === 0)
    .sum();
}
