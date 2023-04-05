// Under value semantics, `const` really does mean constant.

export default function () {
  const nums = [1, 2, 3];
  nums.push(4);
  // ValueScript: TypeError: Cannot mutate this because it is const

  return nums;
  // JavaScript:  [1, 2, 3, 4]
}

// JavaScript allows `nums.push(4)` because it technically doesn't mutate
// `nums`. It mutates the array that `nums` points to instead.
//
// In ValueScript, values never change, so it really is `nums` that changes,
// and therefore `const` is violated. In other words, `const` really does mean
// constant - the value bound to a `const` variable never changes.
//
// ValueScript will reject direct const violations (e.g. assignment) at
// compile-time, but these const violations via methods are more difficult to
// detect, and are currently only detected at run-time. It's a type checking
// problem, and we don't yet do any of that ourselves (we rely on TypeScript for
// type checking). This will improve in the future.
