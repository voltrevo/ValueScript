//! test_output(4782)

import assert from "../helpers/assert.ts";
import Range from "../helpers/Range.ts";

export default function main() {
  // TODO: Remove the temptation pull out this constant (optimization to eval
  // known expressions).
  const threshold = 10n ** 999n;

  const result = Range.from(fibonacci())
    .indexed()
    .filter(([_, x]) => x > threshold)
    .first();

  assert(result !== undefined);

  return result[0];
}

function* fibonacci() {
  let fibLast = 1n;
  let fib = 0n;

  while (true) {
    yield fib;
    [fib, fibLast] = [fib + fibLast, fib];
  }
}
