//! test_output(2)
// Should be: 1

export default function main() {
  const x = Debug.makeCopyCounter("x");

  // Expected copy
  let arr = [x, "y", "z"];

  // Evaluating (arr) here causes it to be stored in a temporary, which forces the mutation below
  // to copy the array.
  arr[1];

  // Shouldn't copy, but does
  arr[2] = "zz";

  return x.count;
}
