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

    // To see progress (takes a few minutes)
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

    if (abundantNumbers.includes(n - abundantNumber)) {
      return true;
    }
  }

  return false;
}
