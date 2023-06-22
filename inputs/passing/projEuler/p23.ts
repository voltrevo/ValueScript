//! test_output_slow(4179871)

import { Range_numbers } from "../helpers/range.ts";
import { properFactorSum } from "./helpers/properFactorSum.ts";

export default function main() {
  const abundantNumbers = [
    ...Range_numbers(1, 28123)
      .filter(isAbundant),
  ];

  return Range_numbers(1, 28123)
    .indexed()
    .flatMap(function* ([_i, n]) {
      // Uncomment to see progress (program takes ~50s with a release build)
      // if (_i % 1000 === 0) {
      //   Debug.log(_i);
      // }

      if (!hasAbundantSum(n, abundantNumbers)) {
        yield n;
      }
    })
    .sum();
}

function isAbundant(n: number) {
  return properFactorSum(n) > n;
}

function hasAbundantSum(n: number, abundantNumbers: number[]) {
  for (let i = 0; i < abundantNumbers.length; i++) {
    const abundantNumber = abundantNumbers[i];

    if (abundantNumber > n) {
      return false;
    }

    if (binarySearch(abundantNumbers, n - abundantNumber)) {
      return true;
    }
  }

  return false;
}

function binarySearch(array: number[], value: number) {
  let min = 0;
  let max = array.length - 1;

  while (min <= max) {
    const mid = Math.floor((min + max) / 2);
    const guess = array[mid];

    if (guess === value) {
      return true;
    }

    if (guess > value) {
      max = mid - 1;
    } else {
      min = mid + 1;
    }
  }

  return false;
}
