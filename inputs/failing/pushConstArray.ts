// test_output! [1,2,3]
// (This is wrong.)

export default function () {
  const arr = [1, 2];
  arr.push(3); // Should throw

  return arr;
}
