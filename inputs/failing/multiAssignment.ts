// test_output! [undefined,1]
// TODO: This is wrong.

export default function main() {
  let a;
  let b;
  a = b = 1;

  return [a, b];
}
