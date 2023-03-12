import plus from "./helpers/plus.ts";

export default function main() {
  return nameListScore([
    /* insert names here */
  ]);
}

function nameListScore(names: string[]) {
  names.sort();

  return names
    .map((name, i) => (i + 1) * nameScore(name))
    .reduce(plus);
}

function nameScore(name: string) {
  let sum = 0;

  for (const c of name) {
    sum += c.charCodeAt(0) - "A".charCodeAt(0) + 1;
  }

  return sum;
}
