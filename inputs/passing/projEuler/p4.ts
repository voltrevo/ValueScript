//! test_output(906609)

export default function main() {
  let largest = 0;

  for (let i = 999; i >= 100; i--) {
    for (let j = i; j >= 100; j--) {
      const product = i * j;

      if (product <= largest) {
        break;
      }

      if (isPalindrome(product)) {
        largest = product;
      }
    }
  }

  return largest;
}

function isPalindrome(n: number) {
  let nStr = `${n}`;
  let lenM1 = nStr.length - 1;
  let halfLen = nStr.length / 2;

  for (let i = 0; i < halfLen; i++) {
    if (nStr[i] !== nStr[lenM1 - i]) {
      return false;
    }
  }

  return true;
}
