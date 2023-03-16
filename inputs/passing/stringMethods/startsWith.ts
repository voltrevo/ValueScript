// test_output! [[true,false,true,false,false,true]]

export default function () {
  const startsWithTests = [
    "abcde".startsWith("abc"),
    "abcde".startsWith("bcd"),
    "abcde".startsWith(""),
    "abcde".startsWith("abcdeabcde"),
    "abcde".startsWith("fgh"),
    "abcde".startsWith("abc", 0),
  ];

  return [startsWithTests];
}
