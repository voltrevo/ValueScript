//! test_output(2)
// Should be: 1

/// <reference path="../../../concept-code/vs.d.ts" />

export default function main() {
  const x = Debug.makeCopyCounter("x");

  let obj: Record<string, unknown> = { x }; // Single copy occurs here
  obj.y = "y"; // Shouldn't copy, but does

  return x.count;
}
