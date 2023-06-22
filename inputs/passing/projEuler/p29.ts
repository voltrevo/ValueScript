//! test_output_slow(9183)

import { Range_numbers } from "../helpers/range.ts";

export default function main() {
  let nums = [];

  for (const a of Range_numbers(2, 101)) {
    for (const b of Range_numbers(2, 101)) {
      nums.push(BigInt(a) ** BigInt(b));
    }
  }

  nums.sort((a, b) => Number(a - b));

  let uniqueCount = 1;

  for (let i = 1; i < nums.length; i++) {
    if (nums[i] != nums[i - 1]) {
      uniqueCount++;
    }
  }

  return uniqueCount;
}
