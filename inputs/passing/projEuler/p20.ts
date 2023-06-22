//! test_output(648)

import range, { Range_numbers } from "../helpers/range.ts";

export default function main() {
  const factorial100 = Range_numbers(2, 101).map(BigInt).bigProduct();

  return range(`${factorial100}`).map(Number).sum();
}
