//! test_output(1)
// Should be: 0

/// <reference path="../../../concept-code/vs.d.ts" />

export default function main() {
  return Debug.makeCopyCounter("x").count;
}
