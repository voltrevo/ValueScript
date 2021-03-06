export default function main() {
  let sum = 0;

  for (let i = 1; i <= 1000; i++) {
    sum += countEligibleLetters(toWords(i));
  }

  return sum;
}

function toWords(n: number): string {
  if (n < 10) {
    // TODO: Constant extraction. This is a good example of where this would be
    // very beneficial. (Instead, there's a giant literal in the bytecode here
    // and it needs to be decoded every time we enter this code path.)
    return [
      'zero',
      'one',
      'two',
      'three',
      'four',
      'five',
      'six',
      'seven',
      'eight',
      'nine',
    ][n];
  }

  if (n < 20) {
    return [
      'ten',
      'eleven',
      'twelve',
      'thirteen',
      'fourteen',
      'fifteen',
      'sixteen',
      'seventeen',
      'eighteen',
      'nineteen',
    ][n - 10];
  }

  if (n < 100) {
    const lastDigit = n % 10;
    const tennerIndex = (n - lastDigit) / 10 - 2;

    const tenner = [
      'twenty',
      'thirty',
      'forty',
      'fifty',
      'sixty',
      'seventy',
      'eighty',
      'ninety',
    ][tennerIndex];

    if (lastDigit === 0) {
      return tenner;
    }

    return `${tenner}-${toWords(lastDigit)}`;
  }

  if (n < 1000) {
    const lastTwoDigits = n % 100;
    const hundreds = (n - lastTwoDigits) / 100;

    let res: string = `${toWords(hundreds)} hundred`;

    if (lastTwoDigits !== 0) {
      res += ` and ${toWords(lastTwoDigits)}`;
    }

    return res;
  }

  if (n === 1000) {
    return 'one thousand';
  }

  panic();
}

function panic(): never {
  return (undefined as any).boom as never;
}

function countEligibleLetters(str: string) {
  let count = 0;

  for (let i = 0; i < str.length; i++) {
    const c = str[i];

    if (c !== ' ' && c !== '-') {
      count++;
    }
  }

  return count;
}
