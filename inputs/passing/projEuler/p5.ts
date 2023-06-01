//! test_output(232792560)

import { factorize } from "./helpers/primes.ts";

declare const Debug: {
  log: (...args: unknown[]) => undefined;
};

export default function main() {
  let n: number[] = [];

  for (let i = 2; i <= 20; i++) {
    n = lcm(n, [...factorize(i)]);
  }

  return n.reduce((a, b) => a * b);
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
      factors = factors.concat(rightFactors);
      return factors;
    }

    if (rightFactors[0] === undefined) {
      factors = factors.concat(leftFactors);
      Debug.log({ factors });
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
