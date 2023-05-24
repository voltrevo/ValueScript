export default function main() {
  return makeRange(1, 51).reduce((a, b) => a + b);
}

function makeRange(start: number, end: number): number[] {
  if (start === end) {
    return [];
  }

  return [start].concat(makeRange(start + 1, end));
}
