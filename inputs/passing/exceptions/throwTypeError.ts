//! test_error(E: TypeError{"message":"Cannot subscript undefined"})

export default function () {
  const arr = undefined;
  const len = arr.length;
}
