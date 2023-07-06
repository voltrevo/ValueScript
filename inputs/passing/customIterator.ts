//! test_output([5,6,7,8,9])

export default function () {
  let vals: number[] = [];

  for (const x of new Range(5, 10)) {
    vals.push(x);
  }

  return vals;
}

class Range {
  constructor(
    public start: number,
    public end: number,
  ) {}

  [Symbol.iterator]() {
    return new RangeIterator(this.start, this.end);
  }
}

class RangeIterator {
  constructor(public value: number, public end: number) {}

  next() {
    const done = this.value >= this.end;
    const res = { value: this.value, done };

    if (!done) {
      this.value++;
    }

    return res;
  }
}
