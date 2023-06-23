// This program solves https://projecteuler.net/problem=2 using the `Range`
// library. Range provides operations on iterables that are very similar to the
// higher order functions on arrays. By using iterables, you can work with
// infinite sequences (eg fibonacci), and the processing is done incrementally
// instead of serializing each step into memory.

import { Range } from "../lib/mod.ts";

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
