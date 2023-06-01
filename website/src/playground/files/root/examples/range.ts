export default function main() {
  return [...range(0, 10)];
}

function* range(start: number, end: number) {
  for (let i = start; i < end; i++) {
    yield i;
  }
}
