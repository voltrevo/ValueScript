export * from "./fooAndBar.ts";
export * from "./circularA.ts";
export * from "./circularB.ts";

export const x = 42;

export function baz() {
  return "baz";
}

// Conflicts with `b` in circularB.ts, but locals have precedence.
// (Conflicts between multiple export*s produce errors.)
export function b() {
  return "b (local)";
}
