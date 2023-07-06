//! test_output("foofoo")
// Should be: "foobar"

export default function () {
  let str = "bar";
  str = `${"foo"}${str}`;

  return str;
}
