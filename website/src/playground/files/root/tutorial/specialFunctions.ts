// We're still working on the standard library for ValueScript, but lots of
// common things are covered.
//
// This program solves https://projecteuler.net/problem=16 by converting 2^1000
// to a string, separating the digits with `Array.from`, converting those digit
// strings to numbers by using `Number` as a function and `.map`, and then doing
// the sum using `.reduce`.

export default function main() {
  return Array.from(`${2n ** 1000n}`)
    .map(Number)
    .reduce((a, b) => a + b);

  // ie 2^1000 = 1071508... (302 digits)
  //    sum    = 1 + 0 + 7 + 1 + 5 + 0 + 8 + ... = 1366
}

// There are two types of these special functions which aren't written in
// ValueScript:
//
// 1. Built-in language functionality (like those above)
// 2. Foreign functions for platform functionality (like fetch, console.log)
//
// So far, we only have type 1, there are no type 2 functions. Yet.
//
// Platform access is extremely important, but getting the language itself right
// is more foundational and is still being actively developed.

export function alternateSolution() {
  const digits = `${2n ** 1000n}`;
  let sum = 0;

  for (let d of digits) {
    sum += Number(d);
  }

  return sum;
}
