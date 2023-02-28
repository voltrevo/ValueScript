// test_output! 25164150

export default function main() {
  return squareOfSum(100) - sumOfSquares(100);
}

function sumOfSquares(n: number) {
  return n * (n + 1) * (2 * n + 1) / 6;
}

function squareOfSum(n: number) {
  return (n * (n + 1) / 2) ** 2;
}
