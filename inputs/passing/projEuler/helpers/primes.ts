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

export function* factorizeAsPowers(n: number): Generator<[number, number]> {
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

  for (const candidate of primeCandidates()) {
    if (
      (candidate % 7) *
          (candidate % 11) *
          (candidate % 13) *
          (candidate % 17) *
          (candidate % 19) *
          (candidate % 23) *
          (candidate % 29) === 0
    ) {
      continue;
    }

    for (const candidateDiv of primeCandidates()) {
      if (candidateDiv * candidateDiv > candidate) {
        yield candidate;
        break;
      }

      if (candidate % candidateDiv === 0) {
        break;
      }
    }
  }
}

export function* primeCandidates() {
  let candidate = 31;

  while (true) {
    yield candidate;
    candidate += 6;
    yield candidate;
    candidate += 4;
    yield candidate;
    candidate += 2;
    yield candidate;
    candidate += 4;
    yield candidate;
    candidate += 2;
    yield candidate;
    candidate += 4;
    yield candidate;
    candidate += 6;
    yield candidate;
    candidate += 2;
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
