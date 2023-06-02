//! test_output(1366)

import range from "../helpers/range.ts";

export default function main() {
  return range(`${2n ** 1000n}`).map(Number).sum();
}
