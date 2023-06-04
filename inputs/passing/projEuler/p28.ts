import range from "../helpers/range.ts";

// There's an analytic solution for this, but that kinda eliminates the need to
// write code altogether. We're showcasing programming techniques here, not
// just writing down the simplest solution.

export default function () {
  const rayStarters = [
    [1, 9, 25],
    [1, 3, 13],
    [1, 5, 17],
    [1, 7, 21],
  ];

  let sum = range(rayStarters)
    .map(([a, b, c]) => range(ray(a, b, c)).limit(501))
    .flatten()
    .sum();

  // The central 1 has been counted 4 times, so subtract 3.
  sum -= 3;

  return sum;
}

function* ray(a: number, b: number, c: number) {
  yield a;
  yield b;
  yield c;

  while (true) {
    const newC = 3 * c - 3 * b + a;
    yield newC;

    [a, b, c] = [b, c, newC];
  }
}
