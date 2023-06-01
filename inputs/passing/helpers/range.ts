export default function range<T = never>(iter?: Iterable<T> | Iterator<T>) {
  return new Range<T>(iter);
}

class Range<T = never> implements Iterable<T> {
  iterable: Iterable<T>;

  constructor(iter?: Iterable<T> | Iterator<T>) {
    if (iter === undefined) {
      this.iterable = [];
    } else if (isIterator(iter)) {
      this.iterable = {
        [Symbol.iterator]: () => iter,
      };
    } else {
      this.iterable = iter;
    }
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
}

function isIterator<T>(iter: Iterable<T> | Iterator<T>): iter is Iterator<T> {
  return (iter as unknown as Record<string, unknown>).next !== undefined;
}
