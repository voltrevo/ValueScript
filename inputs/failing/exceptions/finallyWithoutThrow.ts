//! test_output(E: undefined)
// Should be: "Ok"

// FIXME: This is failing because the optimizer learned to move out of the exception variable. This
// is resulting in a void->undefined conversion (I think), which breaks because we rely on
// throw(void) to not actually throw.

export default function () {
  let result: string;

  try {
  } finally {
    result = "Ok";
  }

  return result;
}
