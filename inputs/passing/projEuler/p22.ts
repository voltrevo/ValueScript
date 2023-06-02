import range from "../helpers/range.ts";
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
  return range(name)
    .map((c) => c.codePointAt(0)! - "A".codePointAt(0)! + 1)
    .sum();
}
