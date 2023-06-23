//! test_output(4613732)

import Range from "../helpers/Range.ts";

export default function main() {
  return Range.from(fibonacci())
    .while((x) => x < 4_000_000)
    .filter((x) => x % 2 === 0)
    .sum();
}

function* fibonacci() {
  let [fibLast, fib] = [0, 1];

  while (true) {
    yield fib;
    [fibLast, fib] = [fib, fibLast + fib];
  }
}
