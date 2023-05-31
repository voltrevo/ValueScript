//! test_output(["  foo","🚀","🚀","🚀","🚀","🚀","🚀🚀","🚀🚀","12abc","123abc","1231abc","12312abc"])

export default function () {
  return [
    "foo".padStart(5),
    "🚀".padStart(3, "🚀"),
    "🚀".padStart(4, "🚀"),
    "🚀".padStart(5, "🚀"),
    "🚀".padStart(6, "🚀"),
    "🚀".padStart(7, "🚀"),
    "🚀".padStart(8, "🚀"),
    "🚀".padStart(9, "🚀"),
    "abc".padStart(5, "123"),
    "abc".padStart(6, "123"),
    "abc".padStart(7, "123"),
    "abc".padStart(8, "123"),
  ];
}
