// test_output! [71,839,1471,6857]

export default function main() {
  return factorize(600851475143);
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
