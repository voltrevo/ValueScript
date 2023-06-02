import { primes } from "../projEuler/helpers/primes.ts";

export default function range<T>(iterable: Iterable<T>) {
  return new Range(iterable);
}

// TODO: Static methods
export function Range_from<T = never>(
  iter?: Iterable<T> | Iterator<T> | (() => Iterable<T>),
) {
  if (iter === undefined) {
    return Range_fromIterable([]);
  }

  if (typeof iter === "function") {
    return Range_fromIterable(iter());
  }

  // TODO: `in` operator
  if (hasKey(iter, Symbol.iterator)) {
    return Range_fromIterable(iter);
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

export function Range_numbers(start: number, end: number) {
  function* res() {
    for (let i = start; i < end; i++) {
      yield i;
    }
  }

  return Range_fromIterable(res());
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

    return Range_fromIterable(res());
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

    for (const x of this.iterable) {
      res += sep;
      res += x;
    }

    return res;
  }

  sum(): T extends number ? number : never {
    let res = 0;

    for (const x of this.iterable) {
      res += x as number;
    }

    return res as T extends number ? number : never;
  }

  product(): T extends number ? number : never {
    let res = 1;

    for (const x of this.iterable) {
      res *= x as number;
    }

    return res as T extends number ? number : never;
  }

  map<MappedT>(fn: (x: T) => MappedT) {
    const iterable = this.iterable;

    function* res() {
      for (const x of iterable) {
        yield fn(x);
      }
    }

    return Range_fromIterable(res());
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

    return Range_fromIterable(res());
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

    return Range_fromIterable(res());
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

    return Range_fromIterable(res());
  }

  append<U>(newItems: Iterable<U>) {
    const iterable = this.iterable;

    function* res() {
      yield* iterable;
      yield* newItems;
    }

    return Range_fromIterable(res());
  }

  prepend<U>(newItems: Iterable<U>) {
    const iterable = this.iterable;

    function* res() {
      yield* newItems;
      yield* iterable;
    }

    return Range_fromIterable(res());
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

    return Range_fromIterable(res());
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

    return Range_fromIterable(res());
  }

  reduce<S>(state: S, fn: (state: S, x: T) => S) {
    for (const x of this.iterable) {
      state = fn(state, x);
    }

    return state;
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
