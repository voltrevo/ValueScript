export default function main() {
  const values = ["a", "b", "c"];

  return [values, reverse(values)];
}

function reverse<T>(arr: T[]) {
  let left = 0;
  let right = arr.length - 1;

  while (left < right) {
    [arr[left], arr[right]] = [arr[right], arr[left]];

    left++;
    right--;
  }

  return arr;

  // This version also works:
  //   arr.reverse();
  //   return arr;
}
