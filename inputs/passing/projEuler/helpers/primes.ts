export function factorize(n: number): number[] {
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

export function nextOddPrime(n: number): number {
  n += 1 + (n % 2); // Next odd number

  while (!isOddPrime(n)) {
    n += 2;
  }

  return n;
}

export function isOddPrime(n: number): boolean {
  let i = 3;

  while (i * i <= n) {
    if (n % i === 0) {
      return false;
    }

    i += 2;
  }

  return true;
}

export class PrimeGenerator {
  pcg: PrimeCandidatesGenerator;

  constructor() {
    this.pcg = new PrimeCandidatesGenerator();
  }

  next() {
    while (true) {
      const candidate = this.pcg.next();

      if (isPrime(candidate)) {
        return candidate;
      }
    }
  }
}

export function isPrime(n: number) {
  let pcg = new PrimeCandidatesGenerator();

  while (true) {
    const pc = pcg.next();

    if (pc * pc > n) {
      return true;
    }

    if (n % pc === 0) {
      return false;
    }
  }
}

export class PrimeCandidatesGenerator {
  gen: Gen235 | GenMod30;

  constructor() {
    this.gen = new Gen235();
  }

  next() {
    let c = this.gen.next();

    if (c !== undefined) {
      return c;
    }

    this.gen = new GenMod30();
    return this.gen.next();
  }
}

class Gen235 {
  nums: number[];
  i: number;

  constructor() {
    this.nums = [2, 3, 5];
    this.i = 0;
  }

  next() {
    return this.nums[this.i++];
  }
}

class GenMod30 {
  nums: number[];
  i: number;

  constructor() {
    this.nums = [1, 7, 11, 13, 17, 19, 23, 29];
    this.i = 1;
  }

  next() {
    const rem = this.i % 8;
    const rounds = (this.i - rem) / 8;
    this.i++;

    return 30 * rounds + this.nums[rem];
  }
}
