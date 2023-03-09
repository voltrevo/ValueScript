import SillyBigInt from "./helpers/SillyBigInt.ts";

export default function main() {
  let fibLast = new SillyBigInt(1);
  let fib = new SillyBigInt(1);
  let fibIndex = 2;

  while (fib.toString().length < 1000) {
    const tmp = fib;
    fib.add(fibLast);
    fibLast = tmp;
    fibIndex++;
  }

  return fibIndex;
}
