// test_output! [0,2,[2,3,5,7,11],[11,31,41,61,71],547,"v","a",undefined]

import Range from "../helpers/Range.ts";
import { primes } from "../projEuler/helpers/primes.ts";

export default function main() {
  return [
    Range.from([]).count(),
    Range.from([1, 2, 3]).limit(2).count(),
    [...Range.from(primes()).limit(5)],
    [...Range.from(primes()).filter((p) => p % 5 === 1).limit(5)],
    Range.from(primes()).at(100),
    Range.from(["abcdefghijklmnopqrstuvwxyz"].at(-5)),
    Range.from(["abcdefghijklmnopqrstuvwxyz"].at(-26)),
    Range.from(["abcdefghijklmnopqrstuvwxyz"].at(-27)),
  ];
}
