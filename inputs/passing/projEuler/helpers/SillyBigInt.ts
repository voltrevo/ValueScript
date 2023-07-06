export default class SillyBigInt {
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

  toString() {
    let str = "";

    for (let i = 0; i < this.data.length - 1; i++) {
      str = `${this.data[i].toString().padStart(15, "0")}${str}`;
    }

    str = `${this.data[this.data.length - 1]}${str}`;

    return str;
  }
}
