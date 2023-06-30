//! test_output(undefined)
// Should be: E: TypeError{"message":"Cannot subscript undefined"}

// FIXME: This is failing because the optimizer is removing the subscript instruction because the
// result is unused. We need to implement throw detection in the optimizer and only remove
// instructions that can throw when we know that they won't throw.

export default function () {
  const arr = undefined;
  const len = arr.length;
}
