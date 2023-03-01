// test_output! [9,9,9]

// This is not correct - it should be [1,4,9]. The problem is that the rhs
// *value* is supposed to be the result of the assignment, but when each rhs is
// compiled %x is used to store the result directly (e.g. op* 1 1 %x), and so %x
// is then used as the result of the expression.
//
// This could be easily fixed by simply creating an extra register for the rhs
// of each assignment and doing an extra mov to get it into the variable
// register. However, that makes the assembly extremely verbose. I'm hoping
// there's a nice way to generate relatively neat and tidy assembly directly,
// but it might make sense in the end, and just deal with it in assembly-level
// optimization.

export default function main() {
  let x = 0;

  return [
    x = 1 * 1,
    x = 2 * 2,
    x = 3 * 3,
  ];
}
