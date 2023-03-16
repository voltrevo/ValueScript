// test_output! [["abc","xyz","a1b2c3","hello world","áéíóú"]]

export default function () {
  const toLowerCaseTests = [
    "ABC".toLowerCase(),
    "XYZ".toLowerCase(),
    "A1B2C3".toLowerCase(),
    "Hello World".toLowerCase(),
    "ÁÉÍÓÚ".toLowerCase(),
  ];

  return [toLowerCaseTests];
}
