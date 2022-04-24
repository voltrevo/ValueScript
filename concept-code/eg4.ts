export default function main() {
  function Counter() {
    this.value = 0;
  }

  Counter.prototype.inc = function() {
    this.value++;
  }

  let counter = new Counter();
  counter.inc();
  counter.inc();
  const counter2 = counter;
  counter.inc();

  return [counter.value, counter2.value];
}
