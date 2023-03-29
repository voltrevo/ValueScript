// ValueScript is like TypeScript without side effects. We achieve this by
// deviating from JavaScript in three key ways.
//
// 3. When `this` changes inside `counter.next()`, the updated `this` value is
//    used to mutate `counter` when the method returns.

export default function main() {
  let counter = new Counter();

  return [counter.next(), counter.next(), counter.next()];
  // JavaScript:  [1, 2, 3]
  // ValueScript: [1, 2, 3]
}

class Counter {
  value = 0;

  next() {
    this.value++;
    return this.value;
  }
}

// This difference is more subtle - the program as a whole behaves the same but
// the implementation is different.
//
// In JavaScript, when `this.value++` updates `this`, it is doing that directly
// on the `counter` object.
//
// In ValueScript, `this` is just a variable, and variable mutation is always
// local. Instead, methods have an implicit extra output for `this` (similar to
// `this` being an implicit input in both languages). That extra output is used
// to mutate the variable on the left side of the dot when the method returns.