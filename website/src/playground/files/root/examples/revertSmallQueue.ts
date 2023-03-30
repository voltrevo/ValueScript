export default function main() {
  let a = new SmallQueue(["item1"]);
  let b = new SmallQueue(["item2", "item3", "item4"]);
  let errors: unknown[] = [];

  try {
    const item = a.pop();
    b.push(item);
  } catch (e) {
    errors.push(e);
  }

  return {
    a,
    b,
    errors,
  };
}

class SmallQueue<T extends {}> {
  items: T[];

  constructor(items: T[]) {
    if (items.length > 3) {
      throw new Error(`${items.length} is too many items for SmallQueue`);
    }

    this.items = items;
  }

  pop(): T {
    const item = this.items.shift();

    if (item === undefined) {
      throw new Error("Cannot pop empty queue");
    }

    return item;
  }

  push(item: T) {
    if (this.items.length >= 3) {
      throw new Error("SmallQueue is already full");
    }

    this.items.push(item);
  }
}
