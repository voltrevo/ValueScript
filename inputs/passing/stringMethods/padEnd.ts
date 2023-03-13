// test_output! ["foo  ","🚀","🚀","🚀","🚀","🚀","🚀🚀","🚀🚀","abc12","abc123","abc1231","abc12312"]

export default function () {
  return [
    "foo".padEnd(5),
    "🚀".padEnd(3, "🚀"),
    "🚀".padEnd(4, "🚀"),
    "🚀".padEnd(5, "🚀"),
    "🚀".padEnd(6, "🚀"),
    "🚀".padEnd(7, "🚀"),
    "🚀".padEnd(8, "🚀"),
    "🚀".padEnd(9, "🚀"),
    "abc".padEnd(5, "123"),
    "abc".padEnd(6, "123"),
    "abc".padEnd(7, "123"),
    "abc".padEnd(8, "123"),
  ];
}
