// Sometimes TypeScript features generate code. ValueScript supports that too.

export default function () {
  const bigFruit = biggerFruit(Fruit.Lemon, Fruit.Mango);
  const point = new Point(3, 4);

  return {
    bigFruit: {
      raw: bigFruit,
      display: Fruit[bigFruit],
      note: isCitrus(bigFruit) ? "citrus" : "not citrus",
    },
    pointDist: point.dist(),
  };
}

// Enums don't exist in JavaScript
enum Fruit {
  Apple,
  Banana,
  Orange,
  Grape,
  Mango,
  Pineapple,
  Watermelon,
  Lemon,
}

class Point {
  constructor(
    // By specifying public here, these parameters are automatically initialized
    // as fields
    public x: number,
    public y: number,
  ) {}

  dist() {
    return Math.sqrt(this.x ** 2 + this.y ** 2);
  }
}

function biggerFruit(left: Fruit, right: Fruit): Fruit {
  const order = [
    Fruit.Grape,
    Fruit.Lemon,
    Fruit.Banana,
    Fruit.Orange,
    Fruit.Apple,
    Fruit.Mango,
    Fruit.Pineapple,
    Fruit.Watermelon,
  ];

  return order.indexOf(left) > order.indexOf(right) ? left : right;
}

function isCitrus(fruit: Fruit): boolean {
  switch (fruit) {
    case Fruit.Apple:
    case Fruit.Banana:
    case Fruit.Grape:
    case Fruit.Mango:
    case Fruit.Pineapple:
    case Fruit.Watermelon:
      return false;

    case Fruit.Orange:
    case Fruit.Lemon:
      return true;
  }

  // TypeScript knows this is unreachable, so it doesn't complain that we're
  // failing to return a boolean. If you comment out one of the cases, it'll
  // emit an error.
}
