import { factorize } from "./helpers/primes.ts";

declare const Debug: {
  log: (...args: unknown[]) => undefined;
};

export default function main() {
  let triNum = 0;

  for (let i = 1;; i++) {
    triNum += i;
    const factorCount = countFactors(triNum);

    if (factorCount > 500) {
      return triNum;
    }
  }
}

function countFactors(n: number): number {
  let count = 1;
  let power = 0;
  let prevFactor = 0;

  for (const factor of factorize(n)) {
    if (factor !== prevFactor) {
      count *= power + 1;
      power = 1;
      prevFactor = factor;
    } else {
      power++;
    }
  }

  return count;
}
