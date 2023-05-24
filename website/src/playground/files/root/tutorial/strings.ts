// We also deviate from JavaScript in a few other ways:
//
// - Strings are utf-8 and partial code points are not allowed

export default function () {
  //                | JavaScript | ValueScript |
  //                ----------------------------
  return [
    "🚀".length, // |          2 |           4 |
    "🚀"[0], //     |   "\ud83d" |         "🚀" |
    "🚀"[1], //     |   "\ude80" |          "" |
    "🚀"[2], //     |  undefined |          "" |
    "🚀"[3], //     |  undefined |          "" |
    "🚀"[4], //     |  undefined |   undefined |
  ];
}

// In JavaScript, "🚀".length is 2 because 🚀 is the 128,640th unicode character,
// so it requires two 16-bit units to encode.
//
// ValueScript uses utf-8, and requires four 8-bit units instead.
//
// Many would like to see "🚀".length be 1, but it's difficult to do that in a
// performant way. If you have a long string, measuring its length by code
// points requires iterating over the entire string. Alternatively, extra
// bookkeeping could also solve that problem, but it still has a cost and
// dramatically increases the complexity of strings.
//
// JavaScript also interprets string indexing to mean a lookup of a 16-bit unit.
// This is a huge problem because it forces the language to include invalid
// unicode in its string representation, such as "\ud83d" and "\ude80" above.
//
// In ValueScript, strings are always valid unicode. We interpret string
// indexing like so:
//
//   1. Lookup the byte at the given index.
//   2. If there isn't a byte at that index, return undefined.
//   3. If there is a character starting at that byte, return it.
//   4. Otherwise, we're inside the string but there isn't a character starting
//      here, so return an empty string.
//
// Also, `.charCodeAt` is about fixed units and not characters, so we don't
// support it. Use `.codePointAt` instead (probably a good habit for JS too).
//
// If you need JS-style strings, let us know - we might add js`uses utf-16` to
// support this.
