// As a dialect of TypeScript, ValueScript is a superset of JavaScript (at the
// syntax level). We don't rely on type annotations in any way, so you can also
// use plain JavaScript if you prefer.

export default function () {
  let n = 37;

  // TypeScript would complain here, since `n` is inferred as `number`.
  n = 'Hello';

  return n;
}

// We might start making use of types or incorporating our own type checking in
// the future (without breaking untyped usage), but currently the idea is to
// just use your regular TypeScript tooling to check your type annotations (if
// you have them).
