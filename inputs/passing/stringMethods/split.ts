// test_output! [["🚀","🍹","a","b","c","£","한","🎨"],["f","","bar"],["","bar"],["foo",""],["one","two","three"]]

export default function () {
  return [
    "🚀🍹abc£한🎨".split(""),
    "foobar".split("o"),
    "foobar".split("foo"),
    "foobar".split("bar"),
    "one two three".split(" "),
  ];
}
