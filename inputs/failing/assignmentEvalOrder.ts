//! test_output({"right":"right"})
// (This is wrong.)
// Note: The reason for the error (at the time of writing) is not actually
// evaluation order but the use of the *register* %key for the left side. `key`
// is correctly 'evaluated' first, but it doesn't get its own register; it's
// just %key.
// For reference, assignmentEvalOrder2.ts tests the actual order of evaluation.

export default function main() {
  const x = {} as any;
  let key = 'left';

  x[key] = (key = 'right');

  return x;
}
