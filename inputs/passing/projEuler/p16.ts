import SillyBigInt from "./helpers/SillyBigInt.ts";

export default function main() {
  let sbi = new SillyBigInt(1);

  for (let pow = 0; pow < 1000; pow++) {
    sbi.add(sbi);
  }

  return sbi.data.map(digitSum).reduce((a, b) => a + b);
}

function digitSum(n: number) {
  const nStr = `${n}`;
  let sum = 0;

  for (let i = 0; i < nStr.length; i++) {
    sum += +nStr[i];
  }

  return sum;
}
