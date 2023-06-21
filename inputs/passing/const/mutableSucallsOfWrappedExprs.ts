//! test_output(4)

export default function main() {
  let counter = new Counter();

  (counter).inc();
  (<Counter> counter).inc();
  counter!.inc();
  (counter as Counter).inc();

  return counter.value;
}

class Counter {
  value = 0;

  inc() {
    this.value++;
  }
}
