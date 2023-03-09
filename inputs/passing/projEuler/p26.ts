export default function () {
  let longestCycle = 0;
  let longestCycleN = 0;

  for (let n = 2; n < 1_000; n++) {
    const cycleLen = reciprocalDigitsData(n).cycle.length;

    if (cycleLen > longestCycle) {
      longestCycle = cycleLen;
      longestCycleN = n;
    }
  }

  return longestCycleN;
}

function reciprocalDigitsData(n: number) {
  let digits: number[] = [];

  let rems: number[] = [10];
  let rem = 10;

  while (true) {
    const digit = Math.floor(rem / n);
    rem -= digit * n;
    rem *= 10;
    digits.push(digit);

    if (rems.includes(rem)) {
      break;
    }

    rems.push(rem);
  }

  let head = "";

  const cycleRem = rem;
  rem = 10;

  while (rem !== cycleRem) {
    const digit = Math.floor(rem / n);
    rem -= digit * n;
    rem *= 10;

    head += digits.shift();
  }

  return {
    head,
    cycle: digits.join(""),
  };
}
