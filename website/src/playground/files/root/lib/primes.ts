export function* factorize(n: number) {
  for (const p of primes()) {
    if (p * p > n) {
      yield n;
      return;
    }

    while (n % p === 0) {
      yield p;
      n /= p;
    }

    if (n === 1) {
      return;
    }
  }
}

export function* factorizeAsPowers(n: number) {
  let factors = factorize(n);

  let currentFactor = factors.next().value;

  if (currentFactor === undefined) {
    return;
  }

  let currentPower = 1;

  for (const factor of factors) {
    if (factor === currentFactor) {
      currentPower += 1;
    } else {
      yield [currentFactor, currentPower];
      currentFactor = factor;
      currentPower = 1;
    }
  }

  yield [currentFactor, currentPower];
}

export function* primes() {
  yield 2;
  yield 3;
  yield 5;
  yield 7;
  yield 11;
  yield 13;
  yield 17;
  yield 19;
  yield 23;
  yield 29;

  let base = 30;
  let offsets = [1, 7, 11, 13, 17, 19, 23, 29];

  while (true) {
    for (const offset of offsets) {
      let candidate = base + offset;

      if (isPrime(candidate)) {
        yield candidate;
      }
    }

    base += 30;
  }
}

export function isPrime(n: number) {
  for (const p of primes()) {
    if (p * p > n) {
      return true;
    }

    if (n % p === 0) {
      return false;
    }
  }
}
