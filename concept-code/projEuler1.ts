export default function main() {
  let sum = 0;

  for (let i = 0; i < 1000; i = i + 1) {
    if (i % 3 === 0 || i % 5 === 0) {
      sum = sum + i;
    }
  }

  return sum;
}
