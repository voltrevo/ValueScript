export default function() {
  let c = Counter();

  return [c.get(), c.get(), c.get()];
}

function Counter() {
  return {
    next: 1,
    get: function() {
      return this.next++;
    },
  };
}
