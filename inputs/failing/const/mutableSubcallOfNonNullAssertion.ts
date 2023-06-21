export default function main() {
  let counter = new Counter();

  counter!.inc();

  return counter.value;
}

class Counter {
  value = 0;

  inc() {
    this.value++;
  }
}
