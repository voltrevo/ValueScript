//! test_output(1)
// Should be: 0

export default function () {
  return measure(1) - measure(0);
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
