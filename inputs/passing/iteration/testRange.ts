// test_output! [0,2,[2,3,5,7,11],[11,31,41,61,71],547]

import range from "../helpers/range.ts";
import { primes } from "../projEuler/helpers/primes.ts";

export default function main() {
  return [
    range([]).count(),
    range([1, 2, 3]).limit(2).count(),
    [...range(primes()).limit(5)],
    [...range(primes()).filter((p) => p % 5 === 1).limit(5)],
    range(primes()).at(100),
  ];
}
