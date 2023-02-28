// test_output! {"leftBowl":["apple","mango"],"rightBowl":["apple","mango","peach"]}

export default function main() {
  const leftBowl = ['apple', 'mango'];

  let rightBowl = leftBowl;
  rightBowl.push('peach');

  return { leftBowl, rightBowl };
}
