import plus from "./helpers/plus.ts";

export default function () {
  return [
    1,
    raySum([1, 9, 25], 501) - 1,
    raySum([1, 3, 13], 501) - 1,
    raySum([1, 5, 17], 501) - 1,
    raySum([1, 7, 21], 501) - 1,
  ].reduce(plus);
}

type QuadraticTriplet = [number, number, number];

/**
 * There's an analytic solution for this, but that kinda eliminates the need to
 * write code altogether. We're showcasing programming techniques here, not
 * just writing down the simplest solution.
 */
function raySum(triplet: QuadraticTriplet, len: number) {
  let sum = triplet.reduce(plus);

  for (let i = 3; i < len; i++) {
    triplet = nextTriplet(triplet);
    sum += triplet[2];
  }

  return sum;
}

function nextTriplet([a, b, c]: QuadraticTriplet): QuadraticTriplet {
  return [
    b,
    c,
    3 * c - 3 * b + a,
  ];
}
