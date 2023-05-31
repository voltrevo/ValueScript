//! test_output([[1,false],[2,false],[3,false],[undefined,true],[undefined,true]])

export default function () {
  const vals = [1, 2, 3];
  let iter = vals.values();

  return [
    iter.next(),
    iter.next(),
    iter.next(),
    iter.next(),
    iter.next(),
  ].map(({ value, done }) => [value, done]);
}
