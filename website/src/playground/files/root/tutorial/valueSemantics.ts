export default function main() {
  const leftBowl = ["apple", "mango"];

  const rightBowl = leftBowl;
  rightBowl.push("peach");

  return leftBowl.includes("peach");
  // TypeScript:  true
  // ValueScript: false
}

// In TypeScript, `leftBowl` and `rightBowl` are the same object, and that
// object changes. In ValueScript, objects are just data, they don't change.
// When you change `rightBowl`, you are changing the *variable* and therefore
// `leftBowl` doesn't change.