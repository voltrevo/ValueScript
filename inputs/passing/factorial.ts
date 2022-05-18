export default function main(n) {
  return factorial(+n);
}

function factorial(n) {
  if (n === 0) {
    return 1;
  }

  return n * factorial(n - 1);
}
