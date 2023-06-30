//! test_output(0)

/// <reference path="../../../concept-code/vs.d.ts" />

export default function main() {
  return measure(true) - measure(false);
}

function measure(doSwap: boolean) {
  const x = Debug.makeCopyCounter("x");

  let arr: unknown[] = [x, "y", "z"];
  arr = swapFn(arr, 1, 2, doSwap);

  return len(arr) + x.count;
}

function swapFn(arr: unknown[], i: number, j: number, doSwap: boolean) {
  if (doSwap) {
    [arr[i], arr[j]] = [arr[j], arr[i]];
  }

  return arr;
}

function len(arr: unknown[]) {
  return arr.length;
}
