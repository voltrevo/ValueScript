// test_output! 4782

export default function main() {
  let fibLast = 1n;
  let fib = 1n;
  let fibIndex = 2;

  // TODO: Remove the temptation pull out this constant (optimization to eval
  // known expressions).
  const threshold = 10n ** 999n;

  while (fib < threshold) {
    [fib, fibLast] = [fib + fibLast, fib];
    fibIndex++;
  }

  return fibIndex;
}
