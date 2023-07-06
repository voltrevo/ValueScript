//! test_output_slow(21124)

import Range from "../helpers/Range.ts";

export default function main() {
  return Range.numbers(1, 1001)
    .map(toWords)
    .map(countEligibleLetters)
    .sum();
}

function toWords(n: number): string {
  if (n < 10) {
    return [
      "zero",
      "one",
      "two",
      "three",
      "four",
      "five",
      "six",
      "seven",
      "eight",
      "nine",
    ][n];
  }

  if (n < 20) {
    return [
      "ten",
      "eleven",
      "twelve",
      "thirteen",
      "fourteen",
      "fifteen",
      "sixteen",
      "seventeen",
      "eighteen",
      "nineteen",
    ][n - 10];
  }

  if (n < 100) {
    const lastDigit = n % 10;
    const tennerIndex = (n - lastDigit) / 10 - 2;

    const tenner = [
      "twenty",
      "thirty",
      "forty",
      "fifty",
      "sixty",
      "seventy",
      "eighty",
      "ninety",
    ][tennerIndex];

    if (lastDigit === 0) {
      return tenner;
    }

    return `${tenner}-${toWords(lastDigit)}`;
  }

  if (n < 1000) {
    const lastTwoDigits = n % 100;
    const hundreds = (n - lastTwoDigits) / 100;

    let res = `${toWords(hundreds)} hundred`;

    if (lastTwoDigits !== 0) {
      res += ` and ${toWords(lastTwoDigits)}`;
    }

    return res;
  }

  if (n === 1000) {
    return "one thousand";
  }

  panic();
}

function panic(): never {
  throw new Error("Something went wrong");
}

function countEligibleLetters(str: string) {
  let count = 0;

  for (let i = 0; i < str.length; i++) {
    const c = str[i];

    if (c !== " " && c !== "-") {
      count++;
    }
  }

  return count;
}
