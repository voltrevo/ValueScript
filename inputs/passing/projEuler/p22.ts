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
  // Obviously we should be using something like charCodeAt instead of this
  // workaround. Unfortunately that's not implemented yet (TODO). There's also
  // the issue that letterMap is currently rebuilt from scratch on every call
  // to nameScore, which also needs to be fixed (TODO).

  const letterMap = {
    A: 1,
    B: 2,
    C: 3,
    D: 4,
    E: 5,
    F: 6,
    G: 7,
    H: 8,
    I: 9,
    J: 10,
    K: 11,
    L: 12,
    M: 13,
    N: 14,
    O: 15,
    P: 16,
    Q: 17,
    R: 18,
    S: 19,
    T: 20,
    U: 21,
    V: 22,
    W: 23,
    X: 24,
    Y: 25,
    Z: 26,
  };

  let sum = 0;

  for (let i = 0; i < name.length; i++) {
    sum += letterMap[name[i] as keyof typeof letterMap];
  }

  return sum;
}
