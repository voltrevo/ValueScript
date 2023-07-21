//! test_output({"x":3,"y":5})

export default function main() {
  return new Point(3, 5);
}

class Point {
  constructor(public x: number, public y: number) {}
}
