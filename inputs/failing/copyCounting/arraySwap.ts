//! test_output(2)
// Should be: 1

export default function main() {
  const x = Debug.makeCopyCounter("x");

  // Expected copy
  let arr = [x, "y", "z"];

  // Shouldn't copy, but does
  [arr[1], arr[2]] = [arr[2], arr[1]];

  return x.count;
}
