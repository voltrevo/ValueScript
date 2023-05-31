//! test_output([[0,3,1,0,0,3],[-1,-1,-1,-1]])

export default function () {
  const positive = [
    "foobar".indexOf("foo"),
    "foobar".indexOf("bar"),
    "foobar".indexOf("oob"),
    "foobar".indexOf(""),
    "foobar".indexOf("foobar"),
    "foobar".indexOf("bar", 1),
  ];

  const negative = [
    "foobar".indexOf("baz"),
    "foobar".indexOf("qux"),
    "foobar".indexOf("oob", 4),
    "foobar".indexOf("foo", 1),
  ];

  return [positive, negative];
}
