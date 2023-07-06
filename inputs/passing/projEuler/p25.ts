//! test_output(4782)

import assert from "../helpers/assert.ts";
import Range from "../helpers/Range.ts";

export default function main() {
  const result = Range.from(fibonacci())
    .indexed()
    // Note: A naive implementation would calculate 10n ** 999n on every
    // iteration. Fortunately, the optimizer picks up this constant expression
    // and replaces it with its actual value.
    .filter(([_, x]) => x > 10n ** 999n)
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
