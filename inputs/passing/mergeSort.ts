export default function main() {
  let x = [7, 18, 9, 11, 16, 3, 8, 2, 5, 4, 6, 14, 15, 17, 10, 12, 1, 13];

  return mergeSort(x, (a, b) => a - b);
}

function mergeSort<T>(vals: T[], cmp: (a: T, b: T) => number): T[] {
  const len = vals.length;

  if (len <= 1) {
    return vals;
  }

  if (len === 2) {
    if (cmp(vals[0], vals[1]) > 0) {
      return [vals[1], vals[0]];
    }

    return vals;
  }

  const mid = vals.length / 2;

  const leftSorted = mergeSort(vals.slice(0, mid), cmp);
  const rightSorted = mergeSort(vals.slice(mid), cmp);

  let res: T[] = [];

  let left = 0;
  const leftLen = leftSorted.length;
  let right = 0;
  const rightLen = rightSorted.length;

  while (left < leftLen && right < rightLen) {
    if (cmp(leftSorted[left], rightSorted[right]) <= 0) {
      res.push(leftSorted[left++]);
    } else {
      res.push(rightSorted[right++]);
    }
  }

  while (left < leftLen) {
    res.push(leftSorted[left++]);
  }

  while (right < rightLen) {
    res.push(rightSorted[right++]);
  }

  return res;
}
