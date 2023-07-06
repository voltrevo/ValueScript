//! test_output(undefined)

export default function () {
  switch (37 as unknown) {
    case 42:
      return "matched 42";
  }
}
