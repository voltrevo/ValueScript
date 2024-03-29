//! test_output(0)

/// <reference path="../../../concept-code/vs.d.ts" />

export default function main() {
  return measure(true) - measure(false);
}

function measure(doPush: boolean) {
  const x = Debug.makeCopyCounter("x");

  let arr: unknown[] = echo([x]);

  if (doPush) {
    arr = push(arr, "y");
  }

  return (0 * len(arr)) + x.count;
}

function push<T>(x: T[], value: T) {
  x.push(value);
  return x;
}

function echo<T>(x: T) {
  return x;
}

function len<T>(arr: T[]) {
  return arr.length;
}
