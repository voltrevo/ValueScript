// test_output! [17,15,12]

export default function main() {
  let a = 2;
  let b = 3;
  let c = 5;
  a += b += c += 7;

  return [a, b, c];
}
