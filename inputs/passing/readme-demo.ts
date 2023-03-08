// test_output! false

export default function main() {
  const leftBowl = ["apple", "mango"];

  let rightBowl = leftBowl;
  rightBowl.push("peach");

  return leftBowl.includes("peach");
  // TypeScript:  true
  // ValueScript: false
}
