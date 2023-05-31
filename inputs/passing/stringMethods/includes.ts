//! test_output([[true,true,true,true,true,true],[false,false,false,false]])

export default function () {
  const positive = [
    "foobar".includes("foo"),
    "foobar".includes("bar"),
    "foobar".includes("oob"),
    "foobar".includes(""),
    "foobar".includes("foobar"),
    "foobar".includes("bar", 1),
  ];

  const negative = [
    "foobar".includes("baz"),
    "foobar".includes("qux"),
    "foobar".includes("oob", 4),
    "foobar".includes("foo", 1),
  ];

  return [positive, negative];
}
