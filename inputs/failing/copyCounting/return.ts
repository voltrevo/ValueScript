//! test_output(1)
// Should be: 0

/// <reference path="../../../concept-code/vs.d.ts" />

export default function main() {
  return get().count;
}

function get() {
  return Debug.makeCopyCounter("x");
}
