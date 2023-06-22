//! test_output(0)

/// <reference path="../../../concept-code/vs.d.ts" />

export default function () {
  return measure(true) - measure(false);
}

function measure(call: boolean) {
  let c = new Counter();

  if (call) {
    c.inc();
  }

  return c.cc.count;
}

class Counter {
  cc = Debug.makeCopyCounter("cc");
  value = 0;

  inc() {
    this.value++;
  }
}
