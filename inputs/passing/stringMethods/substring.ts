//! test_output([["abc","de","abcde","cde","bcd"],["","abc",""],["abc","cde","bcd"]])

export default function () {
  const positive = [
    "abcde".substring(0, 3),
    "abcde".substring(3, 5),
    "abcde".substring(0),
    "abcde".substring(2, 5),
    "abcde".substring(1, 4),
  ];

  const edgeCases = [
    "abcde".substring(3, 3),
    "abcde".substring(-1, 3),
    "abcde".substring(6, 8),
  ];

  const parameterSwapping = [
    "abcde".substring(3, 0),
    "abcde".substring(5, 2),
    "abcde".substring(4, 1),
  ];

  return [positive, edgeCases, parameterSwapping];
}
