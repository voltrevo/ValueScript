// test_output! E: TypeError{"message":"Cannot mutate this because it is const"}

export default function () {
  const arr = [1, 2];
  arr.push(3); // Should throw

  return arr;
}
