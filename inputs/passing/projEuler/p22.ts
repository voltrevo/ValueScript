//! test_output(2970)

import range from "../helpers/range.ts";

export default function main() {
  return nameListScore([
    "MARY",
    "PATRICIA",
    "LINDA",
    "BARBARA",
    "ELIZABETH",
    "JENNIFER",
    "MARIA",
    "SUSAN",
    "MARGARET",
    // Truncated. Expected output is 871198282 for the full list.
  ]);
}

function nameListScore(names: string[]) {
  names.sort();

  return range(names)
    .indexed()
    .map(([i, name]) => (i + 1) * nameScore(name))
    .sum();
}

function nameScore(name: string) {
  return range(name)
    .map((c) => c.codePointAt(0)! - "A".codePointAt(0)! + 1)
    .sum();
}
