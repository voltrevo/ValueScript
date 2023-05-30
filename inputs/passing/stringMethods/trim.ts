//! test_output([["abc","abc","abc","a b c",""]])

export default function () {
  const trimTests = [
    "  abc".trim(),
    "abc  ".trim(),
    "  abc  ".trim(),
    "  a b c  ".trim(),
    "   ".trim(),
  ];

  return [trimTests];
}
