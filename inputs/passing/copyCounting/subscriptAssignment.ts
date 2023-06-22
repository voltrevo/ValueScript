//! test_output(0)

/// <reference path="../../../concept-code/vs.d.ts" />

export default function main() {
  return measure(true) - measure(false);
}

function measure(assign: boolean) {
  const x = Debug.makeCopyCounter("x");

  let obj: Record<string, unknown> = { x };

  if (assign) {
    obj.y = "y";
  }

  let arr: unknown[] = [x];

  if (assign) {
    arr[1] = "y";
  }

  return x.count;
}
