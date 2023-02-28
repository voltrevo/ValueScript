// test_output! 4613732

export default function main() {
  let sum = 0;

  let fa = 0;
  let fb = 1;

  while (true) {
    const f = fa + fb;

    if (f > 4000000) {
      return sum;
    }

    if (f % 2 === 0) {
      sum += f;
    }

    fa = fb;
    fb = f;
  }
}
