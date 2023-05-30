//! test_output(["this is the foo function","this is the bar function"])

import foobar from "./helpers/foobar.ts";

export default function () {
  return [foobar.foo(), foobar.bar()];
}
