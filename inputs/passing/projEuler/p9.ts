// test_output! 31875000

export default function main() {
  for (let a = 1; a <= 1000; a++) {
    for (let b = a; b <= 1000; b++) {
      const c = 1000 - a - b;

      if (c <= b) {
        break;
      }

      if (a * a + b * b === c * c) {
        return a * b * c;
      }
    }
  }
}
