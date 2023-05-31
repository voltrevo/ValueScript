//! test_output(1366)

import plus from "./helpers/plus.ts";

export default function main() {
  return Array.from(`${2n ** 1000n}`).map(Number).reduce(plus);
}
