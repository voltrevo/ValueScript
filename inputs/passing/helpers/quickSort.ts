export default function quickSort<T>(vals: T[], cmp: (a: T, b: T) => number) {
  // Demonstrates the ability to do in-place updates in ValueScript.
  //
  // There's only one reference to `vals`, so we can mutate it in-place
  // without violating value semantics.
  //
  // More on quickSort:
  // https://www.youtube.com/watch?v=Hoixgm4-P4M

  const len = vals.length;
  let ranges: [number, number][] = [[0, len - 1]];

  while (true) {
    const range = ranges.shift();

    if (!range) {
      return vals;
    }

    const [start, end] = range;

    if (end - start <= 0) {
      continue;
    }

    let i = start;
    let j = end;

    let pivotIndex = Math.floor((i + j) / 2);
    [vals[pivotIndex], vals[j]] = [vals[j], vals[pivotIndex]];
    const pivot = vals[j];
    j--;

    while (true) {
      while (cmp(vals[i], pivot) < 0) {
        i++;
      }

      while (cmp(vals[j], pivot) > 0) {
        j--;
      }

      if (i < j) {
        [vals[i], vals[j]] = [vals[j], vals[i]];
        i++;
        j--;
        continue;
      }

      [vals[i], vals[end]] = [vals[end], vals[i]];

      ranges.push([start, i - 1]);
      ranges.push([i + 1, end]);

      break;
    }
  }
}
