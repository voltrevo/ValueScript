//! test_output(648)

import Range from "../helpers/Range.ts";

export default function main() {
  const factorial100 = Range.numbers(2, 101).map(BigInt).bigProduct();

  return Range.from(`${factorial100}`).map(Number).sum();
}
