// test_output! 8

export default function () {
  try {
    throw [3, 5];
  } catch ([a, b]) {
    return a + b;
  }
}
