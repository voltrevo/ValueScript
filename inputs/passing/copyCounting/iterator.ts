//! test_output(0)

export default function () {
  return measure(10) - measure(0);
}

function measure(n: number) {
  const x = Debug.makeCopyCounter("x");

  for (const _ of numbers(x, n)) {
    //
  }

  return x.count;
}

function* numbers(_x: unknown, n: number) {
  for (let i = 0; i < n; i++) {
    yield i;
  }
}
