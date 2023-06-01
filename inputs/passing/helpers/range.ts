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
}

function isIterator<T>(iter: Iterable<T> | Iterator<T>): iter is Iterator<T> {
  return (iter as unknown as Record<string, unknown>).next !== undefined;
}
