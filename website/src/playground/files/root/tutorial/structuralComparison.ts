// Also due to value semantics, arrays and objects are compared on their content instead of their
// identity.

export default function () {
  return new Vec3(-5, 7, 12) === new Vec3(-5, 7, 12);
  // JavaScript:  false
  // ValueScript: true
}

class Vec3 {
  constructor(public x: number, public y: number, public z: number) {}
}

// There's a lot going on to make this work for the underlying functions being compared, especially
// since functions can reference each other recursively. The underlying mechanism is content
// hashing. Check the source or open an issue if you'd like to know more.

// Caveat:
// - TypeScript will emit an error for expressions like `[a, b] === [c, d]`.
//
// This is unfortunate. It's to protect you from accidentally expecting structural comparison in JS,
// but in ValueScript, that's exactly how it *does* work.
//
// ValueScript will still happily evaluate these expressions (ValueScript doesn't use the TS
// compiler), but you might want to rewrite these expressions as `eq([a, b], [c, d])` until
// dedicated ValueScript intellisense is available.
