//! test_output_slow(1366)

import Range from "../helpers/Range.ts";
import SillyBigInt from "./helpers/SillyBigInt.ts";

export default function main() {
  let sbi = new SillyBigInt(1);

  for (let pow = 0; pow < 1000; pow++) {
    sbi.add(sbi);
  }

  return Range.from(sbi.toString())
    .map(Number)
    .sum();
}
