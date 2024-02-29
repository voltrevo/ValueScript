//! test_output(3)

export default function main() {
  let counter = new Counter();

  counter.inc();

  // This syntax breaks when jsx is enabled. It's highly unusual and might be removed from
  // ValueScript entirely.
  // (<Counter> counter).inc();

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
