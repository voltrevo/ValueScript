//! test_output([1,2,3])

export default function main() {
  let c = Counter();

  return [c.get(), c.get(), c.get()];
}

function Counter() {
  return {
    next: 1,
    get: function () {
      return this.next++;
    },
  };
}
