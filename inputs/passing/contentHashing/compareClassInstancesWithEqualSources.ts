//! test_output([true,true,true,true,true,true])

// When functions (and therefore classes) have the same source (and references), they should compare
// equal due to the content hash system.

export default () => {
  const a = new pointClasses.V1(3, 5);
  const b = new pointClasses.V2(3, 5);

  return [
    a === b,
    pointClasses.V1 === pointClasses.V2,
    a instanceof pointClasses.V1,
    a instanceof pointClasses.V2,
    b instanceof pointClasses.V1,
    b instanceof pointClasses.V2,
  ];
};

const pointClasses = {
  V1: class Point {
    constructor(public x: number, public y: number) {}

    lenSq() {
      return this.x ** 2 + this.y ** 2;
    }
  },
  V2: class Point {
    constructor(public x: number, public y: number) {}

    lenSq() {
      return this.x ** 2 + this.y ** 2;
    }
  },
};
