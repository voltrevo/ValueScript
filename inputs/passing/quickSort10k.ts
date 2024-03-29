//! bench()

import quickSort from "./helpers/quickSort.ts";
import randish from "./helpers/randish.ts";
import Range from "./helpers/Range.ts";

export default function main() {
  let nums = [
    ...Range.from(randish())
      .map((x) => Math.floor(8_000 * x))
      .limit(10_000),
  ];

  nums = quickSort(nums, (a, b) => a - b);

  return nums;
}
