// test_output! [14,14,14]

// This is not correct - it should be [1,5,14]. The problem is that the compound
// *value* is supposed to be the result of the assignment, but when these values
// are compiled %x is used to store the result directly (e.g. op+ %x %_tmp0 %x),
// and so %x is then used as the result of the expression.
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
    x += 1 * 1,
    x += 2 * 2,
    x += 3 * 3,
  ];
}
