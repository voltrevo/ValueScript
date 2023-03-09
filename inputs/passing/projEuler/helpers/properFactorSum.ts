import { factorizeAsPowers } from "./primes.ts";

export function properFactorSum(n: number) {
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
