import parallelMap from "./parallelMap";

function isPrime(x: number) {
  if (x <= 2) {
    return x === 2;
  }

  for (let f = 3; f * f <= x; f += 2) {
    if (x % f === 0) {
      return false;
    }
  }

  return true;
}

function Range(limit: number) {
  let res = [];

  for (let i = 0; i < limit; i++) {
    res.push(i);
  }

  return res;
}

function countPrimes(limit: number) {
  let primeFlags = parallelMap(Range(limit), isPrime);

  let count = 0;

  for (const flag of primeFlags) {
    if (flag) {
      count++;
    }
  }

  return count;
}

export default function main() {
  const limit = 1_000_000;
  return `There are ${countPrimes(limit)} primes below ${limit}`;
}
