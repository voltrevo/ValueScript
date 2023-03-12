// test_output! [[NaN,102,111,111,NaN],[102,102,111]]

export default function () {
  return [
    [
      "foo".charCodeAt(-1),
      "foo".charCodeAt(0),
      "foo".charCodeAt(1),
      "foo".charCodeAt(2),
      "foo".charCodeAt(3),
    ],
    [
      "foo".charCodeAt(0 / 0), // (0/0 is NaN, keyword not implemented yet)
      "foo".charCodeAt(-0.9),
      "foo".charCodeAt([1] as any),
    ],
  ];
}
