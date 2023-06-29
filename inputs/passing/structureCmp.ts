//! test_output(24)

export default function () {
  let count = 0;

  const cases: [unknown, unknown, { loose: boolean; strict: boolean }][] = [
    [[], [], { loose: true, strict: true }],
    [[], [1], { loose: false, strict: false }],
    [[1, 2, 3], [1, 2, 3], { loose: true, strict: true }],
    [[1, 2, 3], [1, "2", 3], { loose: true, strict: false }],
    [{}, {}, { loose: true, strict: true }],
    [{}, { x: 1 }, { loose: false, strict: false }],
    [{}, { [Symbol.iterator]: 1 }, { loose: false, strict: false }],
    [{ x: 1, y: 2, z: 3 }, { x: 1, y: 2, z: 3 }, { loose: true, strict: true }],
    [{ x: 1, y: 2, z: 3 }, { x: 1, y: "2", z: 3 }, {
      loose: true,
      strict: false,
    }],
    [[[[[[1]]]]], [[[[[1]]]]], { loose: true, strict: true }],
    [[[[[["1"]]]]], [[[[[1]]]]], { loose: true, strict: false }],
    [null, undefined, { loose: true, strict: false }],
  ];

  for (const [left, right, { loose, strict }] of cases) {
    if ((left == right) === loose) {
      count++;
    } else {
      throw new Error(`Expected ${left} == ${right} to be ${loose}`);
    }

    if ((left === right) === strict) {
      count++;
    } else {
      throw new Error(`Expected ${left} === ${right} to be ${strict}`);
    }
  }

  return count;
}
