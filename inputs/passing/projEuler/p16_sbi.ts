import range from "../helpers/range.ts";
import SillyBigInt from "./helpers/SillyBigInt.ts";

export default function main() {
  let sbi = new SillyBigInt(1);

  for (let pow = 0; pow < 1000; pow++) {
    sbi.add(sbi);
  }

  return range(sbi.toString())
    .map(Number)
    .sum();
}
