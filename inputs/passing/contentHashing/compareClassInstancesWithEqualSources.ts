//! test_output(true)

// When functions (and therefore classes) have the same source (and references), they should compare
// equal due to the content hash system.

export default () => {
  const a = new classes[0](1, 2);
  const b = new classes[1](1, 2);

  return a === b;
};

const classes = [
  class Point {
    constructor(public x: number, public y: number) {}

    lenSq() {
      return this.x ** 2 + this.y ** 2;
    }
  },
  class Point {
    constructor(public x: number, public y: number) {}

    lenSq() {
      return this.x ** 2 + this.y ** 2;
    }
  },
];
