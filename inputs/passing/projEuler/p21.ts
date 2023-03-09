import { factorize } from "./helpers/primes.ts";

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

function factorizeAsPowers(n: number): [number, number][] {
  const factors = factorize(n);

  if (factors.length === 0) {
    return [];
  }

  const result: [number, number][] = [];
  let currentFactor = factors[0];
  let currentPower = 1;

  for (let i = 1; i < factors.length; i++) {
    const factor = factors[i];

    if (factor === currentFactor) {
      currentPower += 1;
    } else {
      result.push([currentFactor, currentPower]);
      currentFactor = factor;
      currentPower = 1;
    }
  }

  result.push([currentFactor, currentPower]);

  return result;
}

function properFactorSum(n: number) {
  const factors = factorizeAsPowers(n);
  return 1 + factorSumMinus1(factors) - n;
}

function factorSumMinus1(factors: [number, number][]): number {
  if (factors.length === 0) {
    return 0;
  }

  const [factor, power] = factors[0];
  let currentFactorSum = 0;

  for (let i = 1; i <= power; i++) {
    currentFactorSum += factor ** i;
  }

  const rest = factors.slice(1);

  return currentFactorSum + (currentFactorSum + 1) * factorSumMinus1(rest);
}
