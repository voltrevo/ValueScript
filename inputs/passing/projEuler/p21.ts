import { Range_numbers } from "../helpers/range.ts";
import { properFactorSum } from "./helpers/properFactorSum.ts";

export default function main() {
  return Range_numbers(2, 10_000)
    .filter(isAmicable)
    .sum();
}

function isAmicable(n: number) {
  const fSum = properFactorSum(n);

  if (fSum === n) {
    return false;
  }

  return properFactorSum(fSum) === n;
}
