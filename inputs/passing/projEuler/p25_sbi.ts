//! test_output_slow(4782)

import assert from "../helpers/assert.ts";
import Range from "../helpers/Range.ts";
import SillyBigInt from "./helpers/SillyBigInt.ts";

export default function main() {
  const res = Range.from(fibonacci())
    .indexed()
    .filter(([_, fib]) => fib.toString().length >= 1000)
    .first();

  assert(res !== undefined);

  return res[0];
}

function* fibonacci() {
  let fibLast = new SillyBigInt(1);
  let fib = new SillyBigInt(0);

  while (true) {
    yield fib;
    const tmp = fib;
    fib.add(fibLast);
    fibLast = tmp;
  }
}
