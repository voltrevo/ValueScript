export default function main() {
  return [...new Range(0, 10)];
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
