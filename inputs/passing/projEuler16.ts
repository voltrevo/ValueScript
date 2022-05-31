export default function main() {
  let sbi = new SillyBigInt(1);
  
  for (let pow = 0; pow < 1000; pow++) {
    sbi.add(sbi);
  }

  return sbi.data.map(digitSum).reduce((a, b) => a + b);
}

class SillyBigInt {
  data: number[];

  constructor(n: number) {
    this.data = [n];
  }

  add(rhs: SillyBigInt) {
    const len = Math.max(this.data.length, rhs.data.length) + 1;
    let carry = 0;

    for (let i = 0; i < len; i++) {
      let sum = carry + (this.data[i] ?? 0) + (rhs.data[i] ?? 0);

      if (sum === 0 && i >= this.data.length) {
        continue;
      }

      if (sum >= 1000000000000000) {
        sum -= 1000000000000000;
        carry = 1;
      } else {
        carry = 0;
      }

      this.data[i] = sum;
    }
  }
}

function digitSum(n: number) {
  const nStr = `${n}`;
  let sum = 0;

  for (let i = 0; i < nStr.length; i++) {
    sum += +nStr[i];
  }

  return sum;
}
