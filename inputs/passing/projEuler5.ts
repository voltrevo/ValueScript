declare const Debug: {
  log: (...args: unknown[]) => undefined;
};

export default function main() {
  let n: number[] = [];

  for (let i = 2; i <= 20; i++) {
    n = lcm(n, factorize(i));
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
