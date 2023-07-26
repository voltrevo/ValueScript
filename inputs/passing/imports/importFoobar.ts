//! test_output(["this is the foo function","this is the bar function"])

import foobar from "./stuff/foobar.ts";

export default function () {
  return [foobar.foo(), foobar.bar()];
}
