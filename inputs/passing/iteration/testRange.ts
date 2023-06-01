// test_output! [0,2,[2,3,5,7,11]]

import range from "../helpers/range.ts";
import { primes } from "../projEuler/helpers/primes.ts";

export default function main() {
  return [
    range().count(),
    range([1, 2, 3]).limit(2).count(),
    [...range(primes()).limit(5)],
  ];
}
