//! test_output(1366)

import Range from "../helpers/Range.ts";

export default function main() {
  return Range.from(`${2n ** 1000n}`).map(Number).sum();
}
