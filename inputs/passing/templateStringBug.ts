//! test_output("foobar")

export default function () {
  let str = "bar";
  str = `${"foo"}${str}`;

  return str;
}
