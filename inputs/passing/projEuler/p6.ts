//! test_output(25164150)

import { Range_numbers } from "../helpers/range.ts";

export default function main() {
  return squareOfSum(100) - sumOfSquares(100);
}

function sumOfSquares(n: number) {
  return Range_numbers(1, n + 1).map((x) => x ** 2).sum();
}

function squareOfSum(n: number) {
  return Range_numbers(1, n + 1).sum() ** 2;
}
