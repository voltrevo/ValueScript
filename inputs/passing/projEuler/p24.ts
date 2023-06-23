//! test_output_slow([2,7,8,3,9,1,5,4,6,0])

import Range from "../helpers/Range.ts";

// TODO: Faster algorithm (it's not necessary to enumerate the permutations).

export default function main() {
  return Range.from(permsOf([0, 1, 2, 3, 4, 5, 6, 7, 8, 9])).at(999_999);
}

function* permsOf(nums: number[]) {
  while (true) {
    yield nums;
    nums = nextPerm(nums);
  }
}

function nextPerm(nums: number[]) {
  let i;

  for (i = nums.length - 2; nums[i] >= nums[i + 1]; i--) {
    if (i === 0) {
      return nums.reverse();
    }
  }

  let j = nums.length - 1;

  while (nums[j] < nums[i]) {
    j--;
  }

  [nums[i], nums[j]] = [nums[j], nums[i]];

  const head = nums.slice(0, i + 1);
  let tail = nums.slice(i + 1);
  tail.sort((a, b) => a - b);

  return [...head, ...tail];
}
