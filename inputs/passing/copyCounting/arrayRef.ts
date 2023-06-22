//! test_output(0)

/// <reference path="../../../concept-code/vs.d.ts" />

export default function main() {
  return measure(true) - measure(false);
}

function measure(ref: boolean) {
  const x = Debug.makeCopyCounter("x");

  let arr = [x, "y", "z"];

  if (ref) {
    // Evaluating (arr) can cause it to be stored in a temporary. If this temporary persists it
    // causes the mutation below to copy.
    arr[1];
  }

  arr[2] = "zz";

  return x.count;
}
