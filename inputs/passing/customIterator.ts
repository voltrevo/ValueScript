//! test_output([5,6,7,8,9])

export default function () {
  let vals: number[] = [];

  for (const x of new Range(5, 10)) {
    vals.push(x);
  }

  return vals;
}

class Range {
  start: number;
  end: number;

  constructor(start: number, end: number) {
    this.start = start;
    this.end = end;
  }

  [Symbol.iterator]() {
    return new RangeIterator(this.start, this.end);
  }
}

class RangeIterator {
  value: number;
  end: number;

  constructor(value: number, end: number) {
    this.value = value;
    this.end = end;
  }

  next() {
    const done = this.value >= this.end;
    const res = { value: this.value, done };

    if (!done) {
      this.value++;
    }

    return res;
  }
}
