// // test_output! 137846528820
// TODO: Regression here... VM says there's an exception now

export default function main() {
  let rc = new RouteCalculator();
  return rc.calculate(20, 20);
}

class RouteCalculator {
  cache: number[][];

  constructor() {
    this.cache = [];
  }

  calculate(i: number, j: number) {
    if (i < 0 || j < 0) {
      return 0;
    }

    if (i === 0 && j === 0) {
      return 1;
    }

    this.cache[i] ??= [];
    let result = this.cache[i][j];

    if (result === undefined) {
      result = this.calculate(i - 1, j) + this.calculate(i, j - 1);
      this.cache[i][j] = result;
    }

    return result;
  }
}
