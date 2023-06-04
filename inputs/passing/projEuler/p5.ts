//! test_output(232792560)

import range, { Range_numbers } from "../helpers/range.ts";
import { factorize } from "./helpers/primes.ts";

export default function main() {
  const factors = Range_numbers(2, 21)
    .map(factorize)
    .reduce([] as number[], (n, i) => lcm(n, [...i]));

  return range(factors).product();
}

function lcm(leftFactors: number[], rightFactors: number[]): number[] {
  let factors: number[] = [];

  while (true) {
    while (leftFactors[0] < rightFactors[0]) {
      factors.push(leftFactors.shift()!);
    }

    while (rightFactors[0] < leftFactors[0]) {
      factors.push(rightFactors.shift()!);
    }

    if (leftFactors[0] === undefined) {
      factors.push(...rightFactors);
      return factors;
    }

    if (rightFactors[0] === undefined) {
      factors.push(...leftFactors);
      return factors;
    }

    let f = leftFactors[0];

    let lPower = 1;
    let rPower = 1;

    leftFactors.shift();
    rightFactors.shift();

    while (leftFactors[0] === f) {
      leftFactors.shift();
      lPower++;
    }

    while (rightFactors[0] === f) {
      rightFactors.shift();
      rPower++;
    }

    const maxPower = Math.max(lPower, rPower);

    for (let i = 0; i < maxPower; i++) {
      factors.push(f);
    }
  }
}
