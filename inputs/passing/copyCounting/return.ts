//! test_output(0)

/// <reference path="../../../concept-code/vs.d.ts" />

export default function main() {
  return get().count;
}

function get() {
  return Debug.makeCopyCounter("x");
}
