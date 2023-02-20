export default function main() {
  let sum = 0;

  const x = 1;

  {
    const x = 2;
    sum += x;
  }

  sum += x;

  return sum; // 3 (not 4)
}
