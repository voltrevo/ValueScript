// Also due to value semantics, arrays and objects are compared on their content instead of their
// identity.

export default function () {
  return vec3(-5, 7, 12) === vec3(-5, 7, 12);
  // JavaScript:  false
  // ValueScript: true
}

function vec3(x: number, y: number, z: number) {
  return { x, y, z };
}

// Caveat:
// - TypeScript will emit an error for expressions like `[a, b] === [c, d]`.
//
// This is unfortunate. It's to protect you from accidentally expecting structural comparison in JS,
// but in ValueScript, that's exactly how it *does* work.
//
// ValueScript will still happily evaluate these expressions (ValueScript doesn't use the TS
// compiler), but you might want to rewrite these expressions as `eq([a, b], [c, d])` until
// dedicated ValueScript intellisense is available.
