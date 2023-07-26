//! test_output([true,true,false,true])

export default () => {
  const a = new Point(1, 2);
  const b = new Point(1, 2);
  const c = new Point(1, 3);

  return [
    a.lenSq === b.lenSq,
    a === b,
    a === c,
    c === c,
  ];
};

class Point {
  constructor(public x: number, public y: number) {}

  lenSq() {
    return this.x ** 2 + this.y ** 2;
  }
}
