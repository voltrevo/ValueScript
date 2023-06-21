export default function* randish() {
  let x = 0.123;

  while (true) {
    x -= (x * x + 1) / (2 * x + 0.1);
    x -= (x * x + 1) / (2 * x + 0.1);
    x -= (x * x + 1) / (2 * x + 0.1);

    const y = 1000 * x;
    yield y - Math.floor(y);
  }
}
