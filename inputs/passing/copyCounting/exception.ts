//! test_output(0)

/// <reference path="../../../concept-code/vs.d.ts" />

export default function () {
  try {
    throwCCEx();
  } catch (e) {
    return e.count;
  }
}

function throwCCEx() {
  while (true) {
    const x = Debug.makeCopyCounter("x");
    throw x;
  }
}
