//! test_output([1,4,9])

export default function () {
  let vals: number[] = [];

  for (const x of [1, 2, 3]) {
    vals.push(x * x);
  }

  return vals;
}
