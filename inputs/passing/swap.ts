// test_output! ["b","a"]

export default function main() {
  const x = ['a', 'b'];

  [x[0], x[1]] = [x[1], x[0]];

  return x;
}
