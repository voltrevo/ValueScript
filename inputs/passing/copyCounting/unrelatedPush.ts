// test_output!(0)

/// <reference path="../../../concept-code/vs.d.ts" />

export default function main() {
  return measure(true) - measure(false);
}

function measure(push: boolean) {
  const x = Debug.makeCopyCounter("x");

  let vals: unknown[] = [x];

  if (push) {
    vals.push("y");
  }

  return x.count;
}
