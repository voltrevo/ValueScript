declare const Debug: {
  log: (...args: unknown[]) => undefined;
};

export default function main() {
  let triNum = 0;

  for (let i = 1; ; i++) {
    triNum += i;
    const factorCount = countFactors(triNum);

    if (factorCount > 500) {
      return triNum;
    }
  }
}

function countFactors(n: number): number {
  const primeFactors = factorize(n);

  let count = 1;
  let power = 0;
  let prevFactor = 0;

  for (let i = 0; i <= primeFactors.length; i++) {
    const factor = primeFactors[i];

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

function factorize(n: number): number[] {
  let factors: number[] = [];
  let p = 2;

  while (true) {
    while (n % p === 0) {
      factors.push(p);
      n /= p;
    }

    if (n === 1) {
      return factors;
    }

    p = nextOddPrime(p);

    if (p * p > n) {
      factors.push(n);
      return factors;
    }
  }
}

function nextOddPrime(n: number): number {
  n += 1 + (n % 2); // Next odd number

  while (!isOddPrime(n)) {
    n += 2;
  }

  return n;
}

function isOddPrime(n: number): boolean {
  let i = 3;

  while (i * i <= n) {
    if (n % i === 0) {
      return false;
    }

    i += 2;
  }

  return true;
}
