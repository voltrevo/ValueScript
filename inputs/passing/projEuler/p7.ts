export default function main() {
  let i = 1;
  let p = 2;

  while (i < 10001) {
    p = nextOddPrime(p);
    i++;
  }

  return p;
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
