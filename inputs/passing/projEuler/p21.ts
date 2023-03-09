import { properFactorSum } from "./helpers/properFactorSum.ts";

export default function main() {
  let amicableNumbers = [];

  for (let i = 2; i < 10_000; i++) {
    if (isAmicable(i)) {
      amicableNumbers.push(i);
    }
  }

  return amicableNumbers.reduce((a, b) => a + b);
}

function isAmicable(n: number) {
  const fSum = properFactorSum(n);

  if (fSum === n) {
    return false;
  }

  return properFactorSum(fSum) === n;
}
