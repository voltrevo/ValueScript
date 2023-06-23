import { primes } from "./primes.ts";

export default class Range<T = never> implements Iterable<T> {
  iterable: Iterable<T>;

  constructor(iterable: Iterable<T>) {
    this.iterable = iterable;
  }

  [Symbol.iterator]() {
    return this.iterable[Symbol.iterator]();
  }

  limit(n: number) {
    const iterable = this.iterable;

    function* res() {
      let i = 0;

      for (const x of iterable) {
        if (i >= n) {
          break;
        }

        yield x;
        i++;
      }
    }

    return new Range(res());
  }

  count() {
    let i = 0;

    for (const _x of this.iterable) {
      i++;
    }

    return i;
  }

  empty() {
    for (const _x of this.iterable) {
      return false;
    }

    return true;
  }

  stringJoin(sep = "") {
    let iter = this[Symbol.iterator]();

    const first = iter.next();

    if (first.done) {
      return "";
    }

    let res = String(first.value);

    for (const x of asIterable(iter)) {
      res += sep;
      res += x;
    }

    return res;
  }

  sum(
    // Warning: ValueScript has a bug where typing the `this` parameter causes it to create a
    // phantom regular parameter. This only works because there aren't any other parameters.
    // TODO: Fix this.
    this: Range<number>,
  ) {
    let res = 0;

    for (const x of this.iterable) {
      res += x;
    }

    return res;
  }

  bigSum(
    // Warning: ValueScript has a bug where typing the `this` parameter causes it to create a
    // phantom regular parameter. This only works because there aren't any other parameters.
    // TODO: Fix this.
    this: Range<bigint>,
  ) {
    let res = 0n;

    for (const x of this.iterable) {
      res += x;
    }

    return res;
  }

  product(this: Range<number>) {
    let res = 1;

    for (const x of this.iterable) {
      res *= x;
    }

    return res;
  }

  bigProduct(
    // Warning: ValueScript has a bug where typing the `this` parameter causes it to create a
    // phantom regular parameter. This only works because there aren't any other parameters.
    // TODO: Fix this.
    this: Range<bigint>,
  ) {
    let res = 1n;

    for (const x of this.iterable) {
      res *= x;
    }

    return res;
  }

  map<MappedT>(fn: (x: T) => MappedT) {
    const iterable = this.iterable;

    function* res() {
      for (const x of iterable) {
        yield fn(x);
      }
    }

    return new Range(res());
  }

  flatMap<MappedT>(fn: (x: T) => Iterable<MappedT>) {
    const iterable = this.iterable;

    function* res() {
      for (const x of iterable) {
        for (const y of fn(x)) {
          yield y;
        }
      }
    }

    return new Range(res());
  }

  flatten<U>(this: Range<Iterable<U>>) {
    const iterable = this.iterable;

    function* res() {
      for (const x of iterable) {
        for (const y of x) {
          yield y;
        }
      }
    }

    return new Range(res());
  }

  filter(fn: (x: T) => boolean) {
    const iterable = this.iterable;

    function* res() {
      for (const x of iterable) {
        if (fn(x)) {
          yield x;
        }
      }
    }

    return new Range(res());
  }

  // TODO: Negative indexes
  at(n: number) {
    let i = 0;

    for (const x of this.iterable) {
      if (i === n) {
        return x;
      }

      i++;
    }
  }

  first() {
    for (const x of this.iterable) {
      return x;
    }
  }

  last() {
    let res: T | undefined;

    for (const x of this.iterable) {
      res = x;
    }

    return res;
  }

  indexed() {
    const iterable = this.iterable;

    function* res() {
      let i = 0;

      for (const x of iterable) {
        yield [i, x] as [number, T];
        i++;
      }
    }

    return new Range(res());
  }

  append<U>(newItems: Iterable<U>) {
    const iterable = this.iterable;

    function* res() {
      yield* iterable;
      yield* newItems;
    }

    return new Range(res());
  }

  prepend<U>(newItems: Iterable<U>) {
    const iterable = this.iterable;

    function* res() {
      yield* newItems;
      yield* iterable;
    }

    return new Range(res());
  }

  zip<U>(other: Iterable<U>) {
    const iterable = this.iterable;

    function* res() {
      let iter1 = iterable[Symbol.iterator]();
      let iter2 = other[Symbol.iterator]();

      while (true) {
        const x1 = iter1.next();
        const x2 = iter2.next();

        if (x1.done || x2.done) {
          break;
        }

        yield [x1.value, x2.value] as [T, U];
      }
    }

    return new Range(res());
  }

  skip(n: number) {
    const iterable = this.iterable;

    function* res() {
      let iter = iterable[Symbol.iterator]();

      for (let i = 0; i < n; i++) {
        iter.next();
      }

      while (true) {
        const x = iter.next();

        if (x.done) {
          break;
        }

        yield x.value;
      }
    }

    return new Range(res());
  }

  reduce<S>(state: S, fn: (state: S, x: T) => S) {
    for (const x of this.iterable) {
      state = fn(state, x);
    }

    return state;
  }

  while(fn: (x: T) => boolean) {
    const iterable = this.iterable;

    function* res() {
      for (const x of iterable) {
        if (fn(x)) {
          yield x;
        } else {
          break;
        }
      }
    }

    return new Range(res());
  }

  window(len: number) {
    const iterable = this.iterable;

    function* res() {
      let iter = iterable[Symbol.iterator]();
      let memory = [];

      for (let i = 0; i < len; i++) {
        const { value, done } = iter.next();

        if (done) {
          return;
        }

        memory.push(value);
      }

      yield new Range(memory);

      let i = 0;

      for (const x of asIterable(iter)) {
        memory[i] = x;

        const memoryCopy = memory;
        const iCopy = i;

        yield new Range((function* () {
          for (let j = 1; j <= len; j++) {
            yield memoryCopy[(iCopy + j) % len];
          }
        })());

        i++;
        i %= len;
      }
    }

    return new Range(res());
  }

  static fromConversion<T = never>(
    iter?: Iterable<T> | Iterator<T> | (() => Iterable<T>),
  ) {
    if (iter === undefined) {
      return new Range([]);
    }

    if (typeof iter === "function") {
      return new Range(iter());
    }

    // TODO: `in` operator
    if (hasKey(iter, Symbol.iterator)) {
      return new Range(iter);
    }

    if (hasKey(iter, "next")) {
      return Range.fromIterator(iter);
    }

    never(iter);
  }

  static from<T = never>(iterable: Iterable<T> = []) {
    return new Range<T>(iterable);
  }

  static fromIterator<T = never>(iterator: Iterator<T>) {
    return new Range<T>({
      [Symbol.iterator]: () => iterator,
    });
  }

  static numbers(start = 0, end?: number) {
    if (end === undefined) {
      return new Range((function* () {
        for (let i = start;; i++) {
          yield i;
        }
      })());
    }

    return new Range((function* () {
      for (let i = start; i < end; i++) {
        yield i;
      }
    })());
  }

  static primes() {
    return new Range(primes());
  }
}

function hasKey<Obj, K extends string | symbol>(
  obj: unknown,
  key: K,
): obj is Obj & Record<K, unknown> {
  return (obj as Record<K, unknown>)[key] !== undefined;
}

function never(x: never): never {
  throw new Error(`Unexpected value: ${x}`);
}

function asIterable<T>(iterator: Iterator<T>): Iterable<T> {
  return { [Symbol.iterator]: () => iterator };
}
