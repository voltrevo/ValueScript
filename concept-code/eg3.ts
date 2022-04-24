export default function main() {
  class Counter {
    value = 0;
    inc() { this.value++; }
  }

  let counter = new Counter();
  counter.inc();
  counter.inc();
  const counter2 = counter;
  counter.inc();

  return [counter.value, counter2.value];
}
