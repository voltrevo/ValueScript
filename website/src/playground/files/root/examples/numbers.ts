export default function main() {
  return [...numbers(0, 10)];
}

function* numbers(start: number, end: number) {
  for (let i = start; i < end; i++) {
    yield i;
  }
}
