// test_output! [["abc","abc  ","abc  ","a b c  ",""]]

export default function () {
  const trimStartTests = [
    "  abc".trimStart(),
    "abc  ".trimStart(),
    "  abc  ".trimStart(),
    "  a b c  ".trimStart(),
    "   ".trimStart(),
  ];

  return [trimStartTests];
}
