//! test_output([true,false,false,true,true,true,false,true])

export default function () {
  return [
    "foo" in { foo: "bar" },
    "bar" in { foo: "bar" },
    "foo" in ["foo"],
    0 in ["foo"],
    "0" in ["foo"],
    "foo" in new C(),
    "forEach" in [],
    "map" in [],
  ];
}

class C {
  foo() {}
}
