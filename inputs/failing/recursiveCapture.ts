export default function main() {
  let one = 1;

  function factorial(n) {
    if (n === 0) {
      return 1;
    }

    return n * factorial(n - one);
  }

  return factorial(5);
}
