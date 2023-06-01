// ValueScript comes with built-in bundling and tree-shaking.
//
// In this example, we import `factorize` from the lib module. In the assembly
// you can see `factorize` has been included, as well as the `primes` and
// `isPrime` functions it depends on.
//
// However, the primes module also defines:
// - `factorizeAsPowers`
//
// These definitions are not included, because the definitions exported by this
// module do not need them. Omitting those unused definitions reduces the
// bytecode for this module from 1,071 to 413 bytes.

import { factorize } from "../lib/mod.ts";

// It's not just the default export that matters. If you uncomment this line,
// the assembly will also include `BinaryTree`, even though it's not used
// anywhere else.
// export { BinaryTree } from "../lib/mod.ts";

export default function main() {
  return [...factorize(18)]; // [2, 3, 3], because 2 * 3 * 3 = 18
}

// These functions are also not in the assembly, because none of our exports use
// them.

// eslint-disable-next-line @typescript-eslint/no-unused-vars
function foo() {
  return bar();
}

function bar() {
  return 0;
}
