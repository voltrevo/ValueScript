import { properFactorSum } from "./helpers/properFactorSum.ts";

export default function main() {
  let abundantNumbers = [];

  for (let i = 1; i < 28123; i++) {
    if (isAbundant(i)) {
      abundantNumbers.push(i);
    }
  }

  let nonAbundantSums = [];

  for (let i = 1; i < 28123; i++) {
    if (!hasAbundantSum(i, abundantNumbers)) {
      nonAbundantSums.push(i);
    }

    // Uncomment to see progress (program takes ~30s with a release build)
    // if (i % 1000 === 0) {
    //   Debug.log(i);
    // }
  }

  return nonAbundantSums.reduce((a, b) => a + b);
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
