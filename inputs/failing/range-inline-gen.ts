//! test_output(E: TypeError{"message":"fn_ is not a function"})

// This is wrong. It should be:
// //(!) test_output(233168)

export default () => {
  return Range
    .numbers(0, 1000)
    .filter((i) => i % 3 === 0 || i % 5 === 0)
    .sum();
};

class Range<T> implements Iterable<T> {
  constructor(
    public iterable: Iterable<T>,
  ) {}

  [Symbol.iterator](): Iterator<T> {
    return this.iterable[Symbol.iterator]();
  }

  static numbers(start: number, end: number) {
    function* gen() {
      for (let i = start; i < end; i++) {
        yield i;
      }
    }

    return new Range(gen());
  }

  filter(shouldInclude: (value: T) => boolean) {
    const iterable = this.iterable;

    return new Range((function* gen() {
      for (const value of iterable) {
        if (shouldInclude(value)) {
          yield value;
        }
      }
    })());
  }

  sum(this: Range<number>): number {
    let res = 0;

    for (const value of this) {
      res += value;
    }

    return res;
  }
}
