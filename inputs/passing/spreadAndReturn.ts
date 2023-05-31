//! test_output({})

export default function () {
  let x = new X();
  [...x];

  return x;
}

class X {
  constructor() {}

  [Symbol.iterator]() {
    return { next: () => ({ value: undefined, done: true }) };
  }
}
