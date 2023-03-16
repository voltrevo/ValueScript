// test_output! [["abc","  abc","  abc","  a b c",""]]

export default function () {
  const trimEndTests = [
    "abc  ".trimEnd(),
    "  abc".trimEnd(),
    "  abc  ".trimEnd(),
    "  a b c  ".trimEnd(),
    "   ".trimEnd(),
  ];

  return [trimEndTests];
}
