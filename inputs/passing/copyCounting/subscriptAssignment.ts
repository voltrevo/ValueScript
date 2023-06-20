//! test_output(2)

/// <reference path="../../../concept-code/vs.d.ts" />

export default function main() {
  const x = Debug.makeCopyCounter("x");

  let obj: Record<string, unknown> = { x }; // First copy
  obj.y = "y"; // No extra copy

  let arr: unknown[] = [x]; // Second copy
  arr[1] = "y"; // No extra copy

  return x.count;
}
