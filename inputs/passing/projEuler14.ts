declare const Debug: {
  log: (...args: unknown[]) => undefined;
};

export default function main() {
  let longest = {
    n: 1,
    len: 1,
  };

  for (let n = 2; n < 1000000; n++) {
    if (n % 50000 === 0) {
      Debug.log(n);
    }

    const len = collatzLen(n);

    if (len > longest.len) {
      longest = { n, len };
    }
  }

  return longest;
}

function collatzLen(n: number) {
  let i = 1;

  while (n !== 1) {
    if (n % 2 === 0) {
      n /= 2;
      i++;
    } else {
      n = (3 * n + 1) / 2;
      i += 2;
    }
  }

  return i;
}
