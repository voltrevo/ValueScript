export default function main() {
  let sum = 0;
  let gen = new PrimeGenerator();

  while (true) {
    const p = gen.next();

    if (p >= 2000000) {
      return sum;
    }

    sum += p;
  }
}

class PrimeGenerator {
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

function isPrime(n: number) {
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

class PrimeCandidatesGenerator {
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
    const rem = this.i % 8
    const rounds = (this.i - rem) / 8;
    this.i++;

    return 30 * rounds + this.nums[rem];
  }
}
