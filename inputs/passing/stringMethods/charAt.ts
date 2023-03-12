// test_output! [["","f","o","o",""],["f","f","o"]]

export default function () {
  return [
    [
      "foo".charAt(-1),
      "foo".charAt(0),
      "foo".charAt(1),
      "foo".charAt(2),
      "foo".charAt(3),
    ],
    [
      "foo".charAt(0 / 0), // (0/0 is NaN, keyword not implemented yet)
      "foo".charAt(-0.9),
      "foo".charAt([1] as any),
    ],
  ];
}
