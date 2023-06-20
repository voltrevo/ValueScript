// test_output!(1)

/// <reference path="../../../concept-code/vs.d.ts" />

export default function main() {
  const x = Debug.makeCopyCounter("x");

  let vals: unknown[] = [x]; // Single copy occurs here
  vals.push("y"); // No extra copy

  return x.count;
}
