//! test_output(["this is the bar function","this is the bar function"])

import { bar, barExported } from "./stuff/bar.ts";

export default function () {
  return [bar(), barExported()];
}
