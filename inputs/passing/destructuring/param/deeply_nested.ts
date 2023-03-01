// test_ouput! 42

export default function main() {
  return foo([[[{ x: { y: [[{ z: 42 }]] } }]]]);
}

function foo([[[{ x: { y: [[{ z }]] } }]]]: [[[{ x: { y: [[{ z: number }]] } }]]]) {
  return z;
}
