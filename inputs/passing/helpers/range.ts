import { primes } from "../projEuler/helpers/primes.ts";

export default function range<T>(iterable: Iterable<T>) {
  return new Range(iterable);
}

// TODO: Static methods
export function Range_from<T = never>(
  iter?: Iterable<T> | Iterator<T> | (() => Iterable<T>),
) {
  if (iter === undefined) {
    return range([]);
  }

  if (typeof iter === "function") {
    return range(iter());
  }

  // TODO: `in` operator
  if (hasKey(iter, Symbol.iterator)) {
    return range(iter);
  }

  if (hasKey(iter, "next")) {
    return Range_fromIterator(iter);
  }

  never(iter);
}

export function Range_fromIterable<T = never>(iterable: Iterable<T> = []) {
  return new Range<T>(iterable);
}

export function Range_fromIterator<T = never>(iterator: Iterator<T>) {
  return new Range<T>({
    [Symbol.iterator]: () => iterator,
  });
}

export function Range_numbers(start = 0, end?: number) {
  if (end === undefined) {
    return range((function* () {
      for (let i = start;; i++) {
        yield i;
      }
    })());
  }

  return range((function* () {
    for (let i = start; i < end; i++) {
      yield i;
    }
  })());
}

export function Range_primes() {
  return new Range(primes());
}

export class Range<T = never> implements Iterable<T> {
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

    return range(res());
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

    return range(res());
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

    return range(res());
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

    return range(res());
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

    return range(res());
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

    return range(res());
  }

  append<U>(newItems: Iterable<U>) {
    const iterable = this.iterable;

    function* res() {
      yield* iterable;
      yield* newItems;
    }

    return range(res());
  }

  prepend<U>(newItems: Iterable<U>) {
    const iterable = this.iterable;

    function* res() {
      yield* newItems;
      yield* iterable;
    }

    return range(res());
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

    return range(res());
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

    return range(res());
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

    return range(res());
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

      yield range(memory);

      let i = 0;

      for (const x of asIterable(iter)) {
        memory[i] = x;

        const memoryCopy = memory;
        const iCopy = i;

        yield range((function* () {
          for (let j = 1; j <= len; j++) {
            yield memoryCopy[(iCopy + j) % len];
          }
        })());

        i++;
        i %= len;
      }
    }

    return range(res());
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
