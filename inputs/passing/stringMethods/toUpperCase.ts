// test_output! [["ABC","XYZ","A1B2C3","HELLO WORLD","ÁÉÍÓÚ"]]

export default function () {
  const toUpperCaseTests = [
    "abc".toUpperCase(),
    "xyz".toUpperCase(),
    "a1b2c3".toUpperCase(),
    "Hello World".toUpperCase(),
    "áéíóú".toUpperCase(),
  ];

  return [toUpperCaseTests];
}
