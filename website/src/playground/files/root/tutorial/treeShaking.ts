// ValueScript comes with built-in bundling and tree-shaking.
//
// In this example, we import `factorize` from the primes module. In the
// assembly, you can see `factorize` has been included, as well as the
// `nextOddPrime` and `isOddPrime` functions it depends on.
//
// However, the primes module also defines:
// - `factorizeAsPowers`
// - `PrimeGenerator`
// - `PrimeCandidatesGenerator`
// - `Gen235`
// - `GenMod30`
//
// These definitions are not included, because the definitions exported by this
// module do not need them. Omitting those unused definitions reduces the
// bytecode for this module from 1,091 to 295 bytes.

import { factorize } from "../lib/primes";

// It's not just the default export that matters. If you uncomment this line,
// the assembly will also include `PrimeCandidatesGenerator`, even though it's
// not used anywhere else.
// export { PrimeCandidatesGenerator } from "../lib/primes";

export default function main() {
  return factorize(18); // [2, 3, 3], because 2 * 3 * 3 = 18
}

// These functions are also not in the assembly, because none of our exports use
// them.
function foo() { return bar(); }
function bar() {}
