//! test_output(["this is the foo function","this is the bar function"])

import { bar, foo } from "./stuff/fooAndBar.ts";

export default function () {
  return [foo(), bar()];
}
