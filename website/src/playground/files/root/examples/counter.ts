export default function main() {
  let c = new Counter();

  return [c.get(), c.get(), c.get()];
}

class Counter {
  next = 1;

  get() {
    return this.next++;
  }
}