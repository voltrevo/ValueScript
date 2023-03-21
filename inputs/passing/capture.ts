// test_output! [1,6,17]

export default function () {
  const poly = buildPoly([1, 2, 3]); // 1 + 2*x + 3*x^2

  return [
    poly(0),
    poly(1),
    poly(2),
  ];
}

type Poly = (x: number) => number;

function pow(n: number): Poly {
  // Captures the number n
  return (x: number) => x ** n;
}

function add(p: Poly, q: Poly): Poly {
  // Captures the functions p and q
  return (x) => p(x) + q(x);
}

function scale(c: number, p: Poly): Poly {
  // Captures the number c and the function p
  return (x) => c * p(x);
}

// Little endian:
// a0*x^0 + a1*x^1 + a2*x^2 + ...
function buildPoly(a: number[]): Poly {
  let poly: Poly = () => 0;

  for (let i = 0; i < a.length; i++) {
    const term = scale(a[i], pow(i));
    poly = add(poly, term);
  }

  return poly;

  // Also works:
  // return a
  //   .map((c, i) => scale(c, pow(i)))
  //   .reduce(add, () => 0);
}
