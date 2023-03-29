// ValueScript is like TypeScript without side effects. We achieve this by
// deviating from JavaScript in three key ways:
//
// 1. Value semantics

export default function main() {
  const leftBowl = ["apple", "mango"];

  let rightBowl = leftBowl;
  rightBowl.push("peach");

  return leftBowl.includes("peach");
  // JavaScript:  true
  // ValueScript: false
}

// In JavaScript, `leftBowl` and `rightBowl` are the same object, and that
// object changes. In ValueScript, objects are just data, they don't change.
// When you change `rightBowl`, you are changing the *variable* and therefore
// `leftBowl` doesn't change.